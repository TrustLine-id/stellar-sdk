#![no_std]

//! Payment Forwarder example.
//!
//! Guards native / SEP-41 transfers with Trustline `require_trustline`.

use soroban_sdk::{contract, contractimpl, token, xdr::ToXdr, Address, Bytes, Env};
use trustline_sdk::{
    encode_call_data, require_trustline_addrs, set_validation_engine,
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

    /// Intent `data` for `pay_native` (frontend / oracle prevalidation).
    ///
    /// Includes `native_token` so the SAC used for the transfer is bound to the
    /// proof (native amount is an explicit argument, not ambient call value).
    pub fn pay_native_intent_data(
        env: Env,
        native_token: Address,
        destination: Address,
        amount: i128,
    ) -> Bytes {
        let args = (native_token, destination, amount).to_xdr(&env);
        encode_call_data(&env, "pay_native", &args)
    }

    /// Intent `data` for `pay_tokens`.
    pub fn pay_tokens_intent_data(
        env: Env,
        destination: Address,
        token: Address,
        amount: i128,
    ) -> Bytes {
        let args = (destination, token, amount).to_xdr(&env);
        encode_call_data(&env, "pay_tokens", &args)
    }

    /// Pay via a Stellar Asset Contract (typically the native XLM SAC).
    pub fn pay_native(
        env: Env,
        sender: Address,
        native_token: Address,
        destination: Address,
        amount: i128,
    ) {
        sender.require_auth();
        assert!(amount > 0, "Invalid amount");

        let data = Self::pay_native_intent_data(
            env.clone(),
            native_token.clone(),
            destination.clone(),
            amount,
        );
        let addresses = soroban_sdk::vec![&env, destination.clone(), native_token.clone()];
        require_trustline_addrs(&env, &sender, amount, &data, &addresses);

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

        let data =
            Self::pay_tokens_intent_data(env.clone(), destination.clone(), token.clone(), amount);
        let addresses = soroban_sdk::vec![&env, destination.clone(), token.clone()];
        require_trustline_addrs(&env, &sender, 0, &data, &addresses);

        token::Client::new(&env, &token).transfer(&sender, &destination, &amount);
    }
}
