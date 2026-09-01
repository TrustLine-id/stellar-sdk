#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

use crate::{encode_call_data, final_tx_id, intent_id, ValidationMode};

#[test]
fn intent_id_is_deterministic() {
    let env = Env::default();
    let sender = Address::generate(&env);
    let protocol = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"pay");

    let a = intent_id(&env, ValidationMode::Dapp, &sender, &protocol, 42, &data);
    let b = intent_id(&env, ValidationMode::Dapp, &sender, &protocol, 42, &data);
    assert_eq!(a, b);
}

#[test]
fn intent_id_changes_when_inputs_change() {
    let env = Env::default();
    let sender = Address::generate(&env);
    let protocol = Address::generate(&env);
    let other_protocol = Address::generate(&env);
    let data = Bytes::from_slice(&env, b"pay");
    let other_data = Bytes::from_slice(&env, b"transfer");

    let base = intent_id(&env, ValidationMode::Dapp, &sender, &protocol, 42, &data);

    assert_ne!(
        base,
        intent_id(&env, ValidationMode::Dapp, &sender, &protocol, 43, &data)
    );
    assert_ne!(
        base,
        intent_id(
            &env,
            ValidationMode::Dapp,
            &sender,
            &protocol,
            42,
            &other_data
        )
    );
    assert_ne!(
        base,
        intent_id(
            &env,
            ValidationMode::Dapp,
            &sender,
            &other_protocol,
            42,
            &data
        )
    );
    assert_ne!(
        base,
        intent_id(
            &env,
            ValidationMode::Dapp,
            &Address::generate(&env),
            &protocol,
            42,
            &data
        )
    );
}

#[test]
fn final_tx_id_is_deterministic() {
    let env = Env::default();
    let id = BytesN::from_array(&env, &[7u8; 32]);
    let timestamp = 1_700_000_000u64;

    let a = final_tx_id(&env, &id, timestamp);
    let b = final_tx_id(&env, &id, timestamp);
    assert_eq!(a, b);
}

#[test]
fn final_tx_id_changes_when_id_or_timestamp_changes() {
    let env = Env::default();
    let id = BytesN::from_array(&env, &[7u8; 32]);
    let other_id = BytesN::from_array(&env, &[8u8; 32]);
    let timestamp = 1_700_000_000u64;

    let base = final_tx_id(&env, &id, timestamp);
    assert_ne!(base, final_tx_id(&env, &id, timestamp + 1));
    assert_ne!(base, final_tx_id(&env, &other_id, timestamp));
}

#[test]
fn encode_call_data_is_deterministic() {
    let env = Env::default();
    let args = Bytes::from_slice(&env, b"\x01\x02");

    let a = encode_call_data(&env, "pay_native", &args);
    let b = encode_call_data(&env, "pay_native", &args);
    assert_eq!(a, b);
}

#[test]
fn encode_call_data_changes_when_name_or_args_change() {
    let env = Env::default();
    let args = Bytes::from_slice(&env, b"\x01\x02");
    let other_args = Bytes::from_slice(&env, b"\x03\x04");

    let base = encode_call_data(&env, "pay_native", &args);
    assert_ne!(base, encode_call_data(&env, "pay_tokens", &args));
    assert_ne!(base, encode_call_data(&env, "pay_native", &other_args));
}
