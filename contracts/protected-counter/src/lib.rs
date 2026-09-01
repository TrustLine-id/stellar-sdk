#![no_std]

//! Demo target for Trustline Firewall (ownership / firewall pattern).
//!
//! Set `admin` to the Trustline Firewall address so privileged calls only
//! succeed when forwarded through the firewall (after Trustline validation).

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contract]
pub struct ProtectedCounter;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Count,
}

#[contractimpl]
impl ProtectedCounter {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Count, &0u32);
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Transfer admin
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    pub fn count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Count).unwrap_or(0)
    }

    /// Privileged bump — requires `admin` auth (the Trustline Firewall).
    pub fn bump(env: Env) -> u32 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        let n: u32 = env.storage().instance().get(&DataKey::Count).unwrap_or(0);
        let next = n + 1;
        env.storage().instance().set(&DataKey::Count, &next);
        next
    }
}

mod test;
