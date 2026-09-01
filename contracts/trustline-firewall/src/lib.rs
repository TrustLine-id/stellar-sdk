#![no_std]

//! Trustline Firewall — access-controlled gateway in front of a third-party contract.
//!
//! Generic intermediary: every forwarded call is Trustline-validated, then
//! invoked on a fixed `target` via CPI (regular call, not delegate).
//!
//! Access model:
//! - `owner` — single admin (`set_target`, `set_owner`, `set_operator`, `set_public_forward`)
//! - `is_operator` — addresses allowed on the protected `forward` path
//! - `public_forward` — when true, any authenticated initiator may `forward` (still Trustline-gated)
//!
//! Callers pass the target function symbol and args explicitly — Soroban has
//! no catch-all fallback. Configure the target so privileged entrypoints
//! require this firewall address (e.g. `admin = firewall`).

use soroban_sdk::{
    contract, contractimpl, contracttype, xdr::ToXdr, Address, Bytes, Env, Symbol, Val, Vec,
};
use trustline_sdk::{encode_call_data, require_trustline, set_validation_engine};

mod events;
use events::{
    FirewallOperatorUpdated, FirewallOwnerUpdated, FirewallPublicForwardUpdated,
    FirewallTargetUpdated,
};

#[contract]
pub struct TrustlineFirewall;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Target,
    Owner,
    PublicForward,
    Operator(Address),
}

#[contractimpl]
impl TrustlineFirewall {
    /// Deploy the firewall in front of `target`.
    ///
    /// Pass the already deployed Validation Engine instance address.
    pub fn __constructor(
        env: Env,
        target: Address,
        validation_engine: Address,
        initial_owner: Address,
        initial_operator: Option<Address>,
        initial_public_forward: bool,
    ) {
        set_validation_engine(&env, &validation_engine);
        env.storage().instance().set(&DataKey::Target, &target);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &initial_owner);
        env.storage()
            .instance()
            .set(&DataKey::PublicForward, &initial_public_forward);
        if let Some(operator) = initial_operator {
            Self::set_operator_flag(&env, &operator, true);
        }
        bump_instance(&env);
    }

    /// Protected contract address.
    pub fn target(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Target).unwrap()
    }

    /// Firewall admin (configuration only — not an operator by default).
    pub fn owner(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).unwrap()
    }

    /// When true, any authenticated initiator may call `forward`.
    pub fn public_forward(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::PublicForward)
            .unwrap_or(false)
    }

    /// Whether `account` may call `forward` when `public_forward` is false.
    pub fn is_operator(env: Env, account: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Operator(account))
            .unwrap_or(false)
    }

    /// Update target — owner only, Trustline-protected.
    pub fn set_target(env: Env, new_target: Address) {
        let owner = Self::require_owner(&env);
        let data = encode_call_data(&env, "set_target", &new_target.clone().to_xdr(&env));
        require_trustline(&env, &owner, 0, &data);
        env.storage().instance().set(&DataKey::Target, &new_target);
        bump_instance(&env);
        FirewallTargetUpdated { new_target }.publish(&env);
    }

    /// Transfer firewall admin — owner only, Trustline-protected.
    pub fn set_owner(env: Env, new_owner: Address) {
        let owner = Self::require_owner(&env);
        let data = encode_call_data(&env, "set_owner", &new_owner.clone().to_xdr(&env));
        require_trustline(&env, &owner, 0, &data);
        env.storage().instance().set(&DataKey::Owner, &new_owner);
        bump_instance(&env);
        FirewallOwnerUpdated {
            old_owner: owner,
            new_owner,
        }
        .publish(&env);
    }

    /// Add or remove an operator allowed on the protected `forward` path.
    pub fn set_operator(env: Env, account: Address, is_operator: bool) {
        let owner = Self::require_owner(&env);
        let payload = (account.clone(), is_operator).to_xdr(&env);
        let data = encode_call_data(&env, "set_operator", &payload);
        require_trustline(&env, &owner, 0, &data);
        Self::set_operator_flag(&env, &account, is_operator);
        bump_instance(&env);
        FirewallOperatorUpdated {
            account,
            is_operator,
        }
        .publish(&env);
    }

    /// Allow or disallow unrestricted initiators on `forward`.
    pub fn set_public_forward(env: Env, enabled: bool) {
        let owner = Self::require_owner(&env);
        let data = encode_call_data(&env, "set_public_forward", &enabled.to_xdr(&env));
        require_trustline(&env, &owner, 0, &data);
        env.storage()
            .instance()
            .set(&DataKey::PublicForward, &enabled);
        bump_instance(&env);
        FirewallPublicForwardUpdated { enabled }.publish(&env);
    }

    /// Pure helper: intent `data` for `set_target`.
    pub fn set_target_intent_data(env: Env, new_target: Address) -> Bytes {
        encode_call_data(&env, "set_target", &new_target.to_xdr(&env))
    }

    /// Pure helper: intent `data` for `set_owner`.
    pub fn set_owner_intent_data(env: Env, new_owner: Address) -> Bytes {
        encode_call_data(&env, "set_owner", &new_owner.to_xdr(&env))
    }

    /// Pure helper: intent `data` for `set_operator`.
    pub fn set_operator_intent_data(env: Env, account: Address, is_operator: bool) -> Bytes {
        let payload = (account, is_operator).to_xdr(&env);
        encode_call_data(&env, "set_operator", &payload)
    }

    /// Pure helper: intent `data` for `set_public_forward`.
    pub fn set_public_forward_intent_data(env: Env, enabled: bool) -> Bytes {
        encode_call_data(&env, "set_public_forward", &enabled.to_xdr(&env))
    }

    /// Pure helper: builds the `data` blob used in the Trustline intent for `forward`.
    pub fn forward_intent_data(env: Env, fn_name: Symbol, args: Vec<Val>) -> Bytes {
        let payload = (fn_name, args).to_xdr(&env);
        encode_call_data(&env, "forward", &payload)
    }

    /// Forward a call to `target` after Trustline validation.
    ///
    /// `initiator` is the business actor for Trustline (`require_trustline` sender) and
    /// must authorize this call. When `public_forward` is false, `initiator` must also
    /// be registered as an operator.
    pub fn forward(env: Env, initiator: Address, fn_name: Symbol, args: Vec<Val>) -> Val {
        Self::require_forward_initiator(&env, &initiator);
        let target: Address = env.storage().instance().get(&DataKey::Target).unwrap();

        let payload = (fn_name.clone(), args.clone()).to_xdr(&env);
        let data = encode_call_data(&env, "forward", &payload);
        require_trustline(&env, &initiator, 0, &data);

        bump_instance(&env);
        env.invoke_contract(&target, &fn_name, args)
    }

    fn require_owner(env: &Env) -> Address {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        owner
    }

    fn require_forward_initiator(env: &Env, initiator: &Address) {
        initiator.require_auth();
        let public: bool = env
            .storage()
            .instance()
            .get(&DataKey::PublicForward)
            .unwrap_or(false);
        if public {
            return;
        }
        let allowed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Operator(initiator.clone()))
            .unwrap_or(false);
        if !allowed {
            panic!("Unauthorized");
        }
    }

    fn set_operator_flag(env: &Env, account: &Address, enabled: bool) {
        let key = DataKey::Operator(account.clone());
        if enabled {
            env.storage().persistent().set(&key, &true);
            bump_persistent(env, &key);
        } else {
            env.storage().persistent().remove(&key);
        }
    }
}

fn bump_instance(env: &Env) {
    let max = env.storage().max_ttl();
    let week = 120_960u32;
    env.storage()
        .instance()
        .extend_ttl(max.saturating_sub(week), max);
}

fn bump_persistent(env: &Env, key: &DataKey) {
    let max = env.storage().max_ttl();
    let week = 120_960u32;
    env.storage()
        .persistent()
        .extend_ttl(key, max.saturating_sub(week), max);
}
