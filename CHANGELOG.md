# Changelog

All notable changes to the **`trustline-sdk`** crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-28

### Added

- Initial release of the **`trustline-sdk`** Soroban integration crate.
- Deterministic intent hashing: `intent_id`, `final_tx_id`.
- `ValidationMode` (currently `Dapp` only).
- Integrator CPI helpers: `require_trustline`, `require_trustline_addrs`, `require_trustline_adv`, `check_trustline_status`, `check_status_addrs`.
- `ValidationEngineClient` for typed VE CPI (`require_*` / `check_*`).
- Instance wiring: `set_validation_engine`, `validation_engine`, `encode_call_data`.
- Unit tests for intent hash determinism and `encode_call_data` stability.
- Example contracts in this repository (not published to crates.io):
  - `payment-forwarder` — guarded native / SEP-41 transfers
  - `trustline-firewall` — access-controlled gateway + generic `forward` (owner / operators / `public_forward`)
  - `protected-counter` — minimal admin-gated target for firewall demos

### Compatibility

- **`soroban-sdk` 27.x** (Stellar Protocol 23 / Soroban v2 WASM target `wasm32v1-none`).

### Notes

- Deploy a Validation Engine instance separately ([stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine)); pass its contract id to `set_validation_engine` at construction.
- End-to-end oracle → guarded invoke tests live in the validation-engine repository.

[0.1.0]: https://github.com/TrustLine-id/stellar-sdk/releases/tag/v0.1.0
