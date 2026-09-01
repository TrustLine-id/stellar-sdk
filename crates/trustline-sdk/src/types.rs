//! Shared Validation Engine types.
//!
//! `TxState` lives in the validation-engine package.

use soroban_sdk::contracttype;

/// Validation mode included in the intent hash domain.
///
/// Only [`ValidationMode::Dapp`] is supported for now. Additional modes may be
/// added later without changing the hashing scheme.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ValidationMode {
    Dapp = 0,
}

impl From<ValidationMode> for u32 {
    fn from(value: ValidationMode) -> Self {
        value as u32
    }
}
