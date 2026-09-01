#![no_std]

//! Test harness contract exercising Trustline SDK guard helpers.

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};
use trustline_sdk::{
    check_trustline_status, encode_call_data, require_trustline, require_trustline_addrs,
    set_validation_engine, validation_engine as read_validation_engine,
};

#[contract]
pub struct TestTrustlinedClient;

#[contractimpl]
impl TestTrustlinedClient {
    /// Store an existing Validation Engine instance (Soroban: no auto-deploy proxy).
    pub fn __constructor(env: Env, validation_engine: Address) {
        set_validation_engine(&env, &validation_engine);
    }

    pub fn validation_engine(env: Env) -> Address {
        read_validation_engine(&env)
    }

    /// Guard with no arguments; `sender` must authorize the call.
    pub fn guarded_no_args(env: Env, sender: Address) {
        sender.require_auth();
        let data = encode_call_data(&env, "guarded_no_args", &Bytes::new(&env));
        require_trustline(&env, &sender, 0, &data);
    }

    /// Guard with an additional target address.
    pub fn guarded_with_address(env: Env, sender: Address, target: Address) {
        sender.require_auth();
        let data = encode_call_data(&env, "guarded_with_address", &Bytes::new(&env));
        let addresses = soroban_sdk::vec![&env, target];
        require_trustline_addrs(&env, &sender, 0, &data, &addresses);
    }

    /// Returns whether the call would pass validation without aborting.
    pub fn can_pass_no_args(env: Env, sender: Address) -> bool {
        let data = encode_call_data(&env, "guarded_no_args", &Bytes::new(&env));
        check_trustline_status(&env, &sender, 0, &data)
    }
}

#[cfg(test)]
mod test;
