//! Thin integration helpers for contracts that embed Trustline checks.
//!
//! Callers pass `sender`, `value`, and `data` explicitly (no ambient call
//! context on Soroban). The helper still:
//! - resolves `protocol` from `env.current_contract_address()`
//! - stores / uses the configured VE address when using [`set_validation_engine`]
//! - performs a single CPI into the Validation Engine

use soroban_sdk::{symbol_short, Address, Bytes, Env, Symbol, Vec};

use crate::client::ValidationEngineClient;
use crate::types::ValidationMode;

/// Instance storage key for the Validation Engine address.
pub const VE_KEY: Symbol = symbol_short!("VE");

/// Store the Validation Engine address on the current contract instance.
///
/// Call from your `__constructor` (or initializer). Deploy the VE instance
/// separately (factory / CLI), then pass its address here.
pub fn set_validation_engine(env: &Env, ve: &Address) {
    env.storage().instance().set(&VE_KEY, ve);
}

/// Read the configured Validation Engine address.
pub fn validation_engine(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&VE_KEY)
        .expect("validation engine not configured")
}

fn protocol(env: &Env) -> Address {
    env.current_contract_address()
}

/// Enforcing call — panics if the intent is not approved.
pub fn require_trustline(env: &Env, sender: &Address, value: i128, data: &Bytes) {
    let ve = validation_engine(env);
    let client = ValidationEngineClient::new(env, &ve);
    client.require_trustline(&protocol(env), sender, &value, data);
}

/// Enforcing call with an address list for sanctions / policy.
pub fn require_trustline_addrs(
    env: &Env,
    sender: &Address,
    value: i128,
    data: &Bytes,
    addresses: &Vec<Address>,
) {
    let ve = validation_engine(env);
    let client = ValidationEngineClient::new(env, &ve);
    client.require_trustline_addrs(&protocol(env), sender, &value, data, addresses);
}

/// Advanced enforcing call with explicit [`ValidationMode`].
pub fn require_trustline_adv(
    env: &Env,
    mode: ValidationMode,
    sender: &Address,
    value: i128,
    data: &Bytes,
    addresses: &Vec<Address>,
) {
    let ve = validation_engine(env);
    let client = ValidationEngineClient::new(env, &ve);
    client.require_trustline_adv(&protocol(env), &mode, sender, &value, data, addresses);
}

/// Non-destructive status query.
pub fn check_trustline_status(env: &Env, sender: &Address, value: i128, data: &Bytes) -> bool {
    let ve = validation_engine(env);
    let client = ValidationEngineClient::new(env, &ve);
    client.check_trustline_status(&protocol(env), sender, &value, data)
}

/// Non-destructive status query with an address list.
pub fn check_status_addrs(
    env: &Env,
    sender: &Address,
    value: i128,
    data: &Bytes,
    addresses: &Vec<Address>,
) -> bool {
    let ve = validation_engine(env);
    let client = ValidationEngineClient::new(env, &ve);
    client.check_status_addrs(&protocol(env), sender, &value, data, addresses)
}

/// Build a stable `data` payload from a function name and argument bytes.
///
/// Integrators can also pass any canonical `Bytes` blob; this helper is an
/// optional convenience (there is no ambient calldata on Soroban).
pub fn encode_call_data(env: &Env, fn_name: &str, args: &Bytes) -> Bytes {
    let mut out = Bytes::new(env);
    let name = Bytes::from_slice(env, fn_name.as_bytes());
    out.append(&name);
    out.append(args);
    out
}
