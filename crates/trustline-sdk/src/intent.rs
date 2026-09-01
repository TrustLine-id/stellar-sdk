//! Deterministic intent / tx id hashing.

use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::types::ValidationMode;

/// Compute the Trustline intent id.
///
/// `sha256` over the XDR encoding of
/// `(network_id, mode, sender, protocol, value, data)`.
/// Backend and on-chain VE must use this exact function for reconciliation.
pub fn intent_id(
    env: &Env,
    mode: ValidationMode,
    sender: &Address,
    protocol: &Address,
    value: i128,
    data: &Bytes,
) -> BytesN<32> {
    let network_id = env.ledger().network_id();
    let mode_u32: u32 = mode.into();
    let payload = (
        network_id,
        mode_u32,
        sender.clone(),
        protocol.clone(),
        value,
        data.clone(),
    );
    env.crypto().sha256(&payload.to_xdr(env)).into()
}

/// Final event / lookup id: `sha256(xdr(id, timestamp))`.
pub fn final_tx_id(env: &Env, id: &BytesN<32>, timestamp: u64) -> BytesN<32> {
    let payload = (id.clone(), timestamp);
    env.crypto().sha256(&payload.to_xdr(env)).into()
}
