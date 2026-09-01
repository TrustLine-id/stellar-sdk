#![cfg(test)]

//! Integration tests for `require_trustline` / `check_trustline_status` wiring.
//!
//! Integrators pass an existing Validation Engine instance via
//! `set_validation_engine` (see `uses_provided_validation_engine_directly`).

use mock_validation_engine::{MockValidationEngine, MockValidationEngineClient};
use protected_counter::ProtectedCounter;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::TestTrustlinedClient;
use super::TestTrustlinedClientClient;

fn deploy_mock_ve(env: &Env, should_pass: bool) -> Address {
    env.register(MockValidationEngine, (should_pass,))
}

fn deploy_client(env: &Env, validation_engine: &Address) -> TestTrustlinedClientClient<'static> {
    let id = env.register(TestTrustlinedClient, (validation_engine,));
    TestTrustlinedClientClient::new(env, &id)
}

#[test]
fn uses_provided_validation_engine_directly() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let target = Address::generate(&env);
    let mock_id = deploy_mock_ve(&env, true);
    let client = deploy_client(&env, &mock_id);

    assert_eq!(client.validation_engine(), mock_id);
    client.guarded_no_args(&sender);
    client.guarded_with_address(&sender, &target);
}

#[test]
#[should_panic(expected = "Not compliant")]
fn guarded_fails_when_engine_rejects() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let mock_id = deploy_mock_ve(&env, true);
    let client = deploy_client(&env, &mock_id);

    MockValidationEngineClient::new(&env, &mock_id).set_should_pass(&false);
    client.guarded_no_args(&sender);
}

#[test]
fn check_trustline_status_reflects_engine() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let mock_id = deploy_mock_ve(&env, true);
    let client = deploy_client(&env, &mock_id);
    let mock = MockValidationEngineClient::new(&env, &mock_id);

    assert!(client.can_pass_no_args(&sender));

    mock.set_should_pass(&false);
    assert!(!client.can_pass_no_args(&sender));
}

#[test]
#[should_panic]
fn rejects_non_validation_engine_at_runtime() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let bad_id = env.register(ProtectedCounter, (&Address::generate(&env),));
    let client = deploy_client(&env, &bad_id);

    client.guarded_no_args(&sender);
}
