//! Contract clients for Validation Engine CPI from integrating contracts.
//!
//! Integrator surface only (`require_*` / `check_*`).
//! Oracle / admin surface (`add_tx`, `get_tx_state`, …) lives in the
//! validation-engine package (`ve_core::ValidationOracleClient`).

use soroban_sdk::{contractclient, Address, Bytes, Env, Vec};

use crate::types::ValidationMode;

/// Integrator CPI surface for the Validation Engine.
///
/// `protocol` is the integrating service contract. SDK helpers fill it via
/// `env.current_contract_address()` and the VE requires `protocol.require_auth()`.
#[contractclient(name = "ValidationEngineClient")]
pub trait ValidationEngine {
    fn require_trustline(env: Env, protocol: Address, sender: Address, value: i128, data: Bytes);

    fn require_trustline_addrs(
        env: Env,
        protocol: Address,
        sender: Address,
        value: i128,
        data: Bytes,
        addresses: Vec<Address>,
    );

    fn require_trustline_adv(
        env: Env,
        protocol: Address,
        mode: ValidationMode,
        sender: Address,
        value: i128,
        data: Bytes,
        addresses: Vec<Address>,
    );

    fn check_trustline_status(
        env: Env,
        protocol: Address,
        sender: Address,
        value: i128,
        data: Bytes,
    ) -> bool;

    fn check_status_addrs(
        env: Env,
        protocol: Address,
        sender: Address,
        value: i128,
        data: Bytes,
        addresses: Vec<Address>,
    ) -> bool;

    fn check_status_adv(
        env: Env,
        protocol: Address,
        mode: ValidationMode,
        sender: Address,
        value: i128,
        data: Bytes,
        addresses: Vec<Address>,
    ) -> bool;
}
