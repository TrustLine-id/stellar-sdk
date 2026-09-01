#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn bump_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register(ProtectedCounter, (&admin,));
    let client = ProtectedCounterClient::new(&env, &id);
    assert_eq!(client.bump(), 1);
    assert_eq!(client.count(), 1);
}

#[test]
fn set_admin_transfers_privilege() {
    let env = Env::default();
    env.mock_all_auths();
    let deployer = Address::generate(&env);
    let firewall = Address::generate(&env);
    let id = env.register(ProtectedCounter, (&deployer,));
    let client = ProtectedCounterClient::new(&env, &id);
    client.set_admin(&firewall);
    assert_eq!(client.admin(), firewall);
}
