#![no_std]

//! Payment Forwarder example.
//!
//! Guards native / SEP-41 transfers with Trustline `require_trustline_addrs!`.

use soroban_sdk::{contract, contractimpl, token, Address, Env};
use trustline_sdk::{
    require_trustline_addrs, set_validation_engine,
    validation_engine as read_validation_engine,
};

#[contract]
pub struct PaymentForwarder;

#[contractimpl]
impl PaymentForwarder {
    /// Pass the deployed Validation Engine instance address.
    pub fn __constructor(env: Env, validation_engine: Address) {
        set_validation_engine(&env, &validation_engine);
    }

    pub fn validation_engine(env: Env) -> Address {
        read_validation_engine(&env)
    }

    /// Pay via a Stellar Asset Contract (typically the native XLM SAC).
    ///
    /// Intent `data` binds `native_token` into the proof (native amount is an
    /// explicit argument, not ambient call value).
    pub fn pay_native(
        env: Env,
        sender: Address,
        native_token: Address,
        destination: Address,
        amount: i128,
    ) {
        sender.require_auth();
        assert!(amount > 0, "Invalid amount");

        require_trustline_addrs!(
            env,
            sender,
            amount,
            "pay_native"(native_token, destination, amount),
            [destination, native_token],
        );

        token::Client::new(&env, &native_token).transfer(&sender, &destination, &amount);
    }

    /// Pay SEP-41 tokens.
    pub fn pay_tokens(
        env: Env,
        sender: Address,
        destination: Address,
        token: Address,
        amount: i128,
    ) {
        sender.require_auth();
        assert!(amount > 0, "Invalid amount");

        require_trustline_addrs!(
            env,
            sender,
            0,
            "pay_tokens"(destination, token, amount),
            [destination, token],
        );

        token::Client::new(&env, &token).transfer(&sender, &destination, &amount);
    }
}
