#![no_std]

//! # Trustline Soroban SDK
//!
//! Thin integration layer for Trustline's Validation Engine on Soroban:
//! types, intent hashing, and CPI helpers with Stellar-native auth and storage.

pub mod client;
pub mod intent;
pub mod trustlined;
pub mod types;

pub use client::ValidationEngineClient;
pub use intent::{final_tx_id, intent_id};
pub use trustlined::{
    check_status_addrs, check_trustline_status, encode_call_data, require_trustline,
    require_trustline_addrs, require_trustline_adv, set_validation_engine, validation_engine,
    VE_KEY,
};
pub use types::ValidationMode;

#[cfg(test)]
mod test;
