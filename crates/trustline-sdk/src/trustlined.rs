//! Thin integration helpers for contracts that embed Trustline checks.
//!
//! Callers pass `sender`, `value`, and `data` explicitly (no ambient call
//! context on Soroban). The helper still:
//! - resolves `protocol` from `env.current_contract_address()`
//! - stores / uses the configured VE address when using [`set_validation_engine`]
//! - performs a single CPI into the Validation Engine

use soroban_sdk::{symbol_short, xdr::ToXdr, Address, Bytes, Env, Symbol, Vec};

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

/// Enforcing call with a pre-built `data` blob — panics if not approved.
///
/// Prefer `require_trustline!` so the action name + args stay next to the check.
pub fn require_trustline_raw(env: &Env, sender: &Address, value: i128, data: &Bytes) {
    let ve = validation_engine(env);
    let client = ValidationEngineClient::new(env, &ve);
    client.require_trustline(&protocol(env), sender, &value, data);
}

/// Enforcing call with a pre-built `data` blob and an address list for sanctions / policy.
///
/// Prefer `require_trustline_addrs!` when you can name the action inline.
pub fn require_trustline_addrs_raw(
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

/// Advanced enforcing call with explicit [`ValidationMode`] and a pre-built `data` blob.
///
/// Prefer `require_trustline_adv!` when you can name the action inline.
pub fn require_trustline_adv_raw(
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

/// `encode_call_data(fn_name, args.to_xdr())` — preferred when args are typed values.
///
/// Prefer `require_trustline!` / `require_trustline_addrs!` / `require_trustline_adv!` so the
/// action name + args stay next to the Trustline check. The backend builds the same `data`
/// from structured `functionPrototype` + positional `args`.
pub fn encode_intent(env: &Env, fn_name: &str, args: impl ToXdr) -> Bytes {
    encode_call_data(env, fn_name, &args.to_xdr(env))
}

/// `encode_intent` + `require_trustline_raw` in one step.
///
/// ```ignore
/// require_trustline!(env, owner, 0, "set_target"(new_target));
/// require_trustline!(env, sender, 0, "noop"());
/// ```
#[macro_export]
macro_rules! require_trustline {
    (
        $env:expr,
        $sender:expr,
        $value:expr,
        $action:literal ( $($arg:ident),* $(,)? )
        $(,)?
    ) => {{
        let __trustline_data = $crate::__encode_intent_args!($env, $action; $($arg),*);
        $crate::require_trustline_raw(&$env, &$sender, $value, &__trustline_data);
    }};
}

/// `encode_intent` + `require_trustline_addrs_raw` in one step.
///
/// ```ignore
/// require_trustline_addrs!(
///     env, sender, amount,
///     "pay_native"(native_token, destination, amount),
///     [destination, native_token],
/// );
/// ```
#[macro_export]
macro_rules! require_trustline_addrs {
    (
        $env:expr,
        $sender:expr,
        $value:expr,
        $action:literal ( $($arg:ident),* $(,)? ),
        [ $($addr:ident),* $(,)? ]
        $(,)?
    ) => {{
        let __trustline_data = $crate::__encode_intent_args!($env, $action; $($arg),*);
        let __trustline_addrs = ::soroban_sdk::vec![&$env, $($addr.clone()),*];
        $crate::require_trustline_addrs_raw(
            &$env,
            &$sender,
            $value,
            &__trustline_data,
            &__trustline_addrs,
        );
    }};
}

/// `encode_intent` + `require_trustline_adv_raw` in one step.
///
/// ```ignore
/// require_trustline_adv!(
///     env, ValidationMode::Dapp, sender, amount,
///     "pay_native"(native_token, destination, amount),
///     [destination, native_token],
/// );
/// ```
#[macro_export]
macro_rules! require_trustline_adv {
    (
        $env:expr,
        $mode:expr,
        $sender:expr,
        $value:expr,
        $action:literal ( $($arg:ident),* $(,)? ),
        [ $($addr:ident),* $(,)? ]
        $(,)?
    ) => {{
        let __trustline_data = $crate::__encode_intent_args!($env, $action; $($arg),*);
        let __trustline_addrs = ::soroban_sdk::vec![&$env, $($addr.clone()),*];
        $crate::require_trustline_adv_raw(
            &$env,
            $mode,
            &$sender,
            $value,
            &__trustline_data,
            &__trustline_addrs,
        );
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __encode_intent_args {
    ($env:expr, $action:literal; ) => {
        $crate::encode_call_data(&$env, $action, &::soroban_sdk::Bytes::new(&$env))
    };
    ($env:expr, $action:literal; $arg:ident) => {
        $crate::encode_intent(&$env, $action, $arg.clone())
    };
    ($env:expr, $action:literal; $($arg:ident),+) => {
        $crate::encode_intent(&$env, $action, ($($arg.clone()),+))
    };
}
