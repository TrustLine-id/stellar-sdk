#![no_std]

//! Minimal Validation Engine mock for SDK integration tests.
//!
//! Configurable pass/fail on `require_trustline*` / `check_*` without oracle proofs.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, Vec};
use trustline_sdk::ValidationMode;

#[contract]
pub struct MockValidationEngine;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    ShouldPass,
}

#[contractimpl]
impl MockValidationEngine {
    pub fn __constructor(env: Env, should_pass: bool) {
        env.storage()
            .instance()
            .set(&DataKey::ShouldPass, &should_pass);
    }

    pub fn set_should_pass(env: Env, should_pass: bool) {
        env.storage()
            .instance()
            .set(&DataKey::ShouldPass, &should_pass);
    }

    pub fn should_pass(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::ShouldPass)
            .unwrap_or(true)
    }

    pub fn require_trustline(
        env: Env,
        protocol: Address,
        _sender: Address,
        _value: i128,
        _data: Bytes,
    ) {
        protocol.require_auth();
        assert!(Self::should_pass(env), "Not compliant");
    }

    pub fn require_trustline_addrs(
        env: Env,
        protocol: Address,
        _sender: Address,
        _value: i128,
        _data: Bytes,
        _addresses: Vec<Address>,
    ) {
        protocol.require_auth();
        assert!(Self::should_pass(env), "Not compliant");
    }

    pub fn require_trustline_adv(
        env: Env,
        protocol: Address,
        _mode: ValidationMode,
        _sender: Address,
        _value: i128,
        _data: Bytes,
        _addresses: Vec<Address>,
    ) {
        protocol.require_auth();
        assert!(Self::should_pass(env), "Not compliant");
    }

    pub fn check_trustline_status(
        env: Env,
        _protocol: Address,
        _sender: Address,
        _value: i128,
        _data: Bytes,
    ) -> bool {
        Self::should_pass(env)
    }

    pub fn check_status_addrs(
        env: Env,
        _protocol: Address,
        _sender: Address,
        _value: i128,
        _data: Bytes,
        _addresses: Vec<Address>,
    ) -> bool {
        Self::should_pass(env)
    }

    pub fn check_status_adv(
        env: Env,
        _protocol: Address,
        _mode: ValidationMode,
        _sender: Address,
        _value: i128,
        _data: Bytes,
        _addresses: Vec<Address>,
    ) -> bool {
        Self::should_pass(env)
    }
}
