# Trustline Stellar SDK

> Part of the Trustline Stellar / Soroban stack, developed with support from the [Stellar Community Fund](https://communityfund.stellar.org) (**SCF #44**).

A Rust / Soroban SDK for protecting Stellar smart contracts from unauthorized access and malicious transactions by integrating Trustline's Oracle with on-chain data sources.

## Features

- ✅ **Transaction Validation** — Validate invocations with customizable off-chain policies
- ✅ **Sanctions Checking** — Verify addresses against an on-chain sanctions list
- ✅ **Validation Modes** — Intent hash domain; currently **`Dapp` only** (extensible later)
- ✅ **Address Verification** — Check sender and related addresses for compliance
- ✅ **Thin CPI helpers** — `require_trustline!` / `require_trustline_addrs!` / `require_trustline_adv!` (+ low-level `*_raw`)
- ✅ **SEP-41 / native SAC examples** — Payment Forwarder for guarded transfers
- ✅ **Firewall gateway example** — Trustline Firewall with owner / operators / `public_forward` + generic `forward`
- ✅ **Flexible Integration** — Point at any deployed Validation Engine instance

## Installation

Add the SDK crate to your contract:

```toml
[dependencies]
trustline-sdk = "0.1"
soroban-sdk = "27"
```

Or depend on a Git tag:

```toml
trustline-sdk = { git = "https://github.com/TrustLine-id/stellar-sdk", tag = "v0.1.0" }
```

See [CHANGELOG.md](CHANGELOG.md) for release notes.

## Architecture

Validation is performed through a small set of on-chain/off-chain components:

- **Your contract** — Calls `require_trustline!` / `require_trustline_addrs!` / `require_trustline_adv!` (or low-level `*_raw`) before sensitive operations. It holds the address of a **Validation Engine instance**.
- **Validation Engine instance** — Per-client deployed contract (typically `TrustlineOracleVE`). Upgradeable via `upgrade(new_wasm_hash)` by the instance admin. Internally talks to the Trustline Registry and an optional sanctions list.
- **Trustline Registry** — Shared Trustline-controlled contract (in [stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine), not this SDK): oracle allowlist + key → address resolution (e.g. sanctions lists).
- **Trustline's Oracle backend** — Off-chain service that publishes proofs with `add_tx(oracle, …)` (`oracle.require_auth()` + `registry.is_oracle`).
- **Sanctions list** — Optional contract exposing `is_sanctioned(Address) -> bool`, registered under a registry key and resolved by the VE.

In short: your contract → Validation Engine instance → (registry + optional sanctions) + Oracle backend.

You deploy (or reuse) a VE instance separately, then pass its address into your contract `__constructor` via `set_validation_engine`.

## Quick Start

### Basic contract integration

```rust
#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};
use trustline_sdk::{
    require_trustline, require_trustline_addrs, set_validation_engine,
};

#[contract]
pub struct MyContract;

#[contractimpl]
impl MyContract {
    pub fn __constructor(env: Env, validation_engine: Address) {
        set_validation_engine(&env, &validation_engine);
    }

    pub fn transfer(env: Env, sender: Address, amount: i128) {
        sender.require_auth();
        require_trustline!(env, sender, amount, "transfer"());
        // … business logic …
    }

    pub fn transfer_to(env: Env, sender: Address, recipient: Address, amount: i128) {
        sender.require_auth();
        require_trustline_addrs!(
            env, sender, amount,
            "transfer_to"(recipient),
            [recipient],
        );
        // … business logic …
    }
}
```

### Using an existing Validation Engine instance

Pass a previously deployed VE contract id into `__constructor` / `set_validation_engine`. There is no auto-deploy of the engine from the SDK — deploy the VE (or use Trustline’s production instance) first, then wire its address.

### Integration constraints

#### Required

- Call `set_validation_engine` once at construction (or an equivalent initializer).
- Pass explicit `sender`, `value`, and `data` into Trustline helpers — Soroban has no ambient call context for those fields.
- Call `sender.require_auth()` (or the appropriate auth) in your contract **before** enforcing Trustline on that sender.

#### Recommended

- Keep `data` canonical and stable for each protected method (it is part of the intent id). Prefer `require_trustline!` / `require_trustline_addrs!` with an inline action name so encoding stays next to the check; the backend rebuilds the same bytes from structured `functionPrototype` + args.
- Include recipients / tokens in the `addresses` list of `require_trustline_addrs!` when they matter to policy or sanctions.
- Prefer upgrading the **VE instance** over forking validation logic into the dapp WASM.

## API Reference

### Helpers

`require_trustline*!` / `require_trustline_*_raw` and `check_*` serve different roles:

| Helper | Role | Enforces? |
|--------|------|-----------|
| `require_trustline!(…)` | Encode action + args, then enforce | Yes |
| `require_trustline_addrs!(…)` | Same + address list for sanctions / policy | Yes |
| `require_trustline_adv!(…)` | Same + explicit `ValidationMode` | Yes |
| `require_trustline_raw(...)` | Enforce with a pre-built `data` blob | Yes |
| `require_trustline_addrs_raw(...)` | Same + address list | Yes |
| `require_trustline_adv_raw(...)` | Same + explicit `ValidationMode` | Yes |
| `check_trustline_status(...)` | Query only | No |
| `check_status_addrs(...)` | Query + addresses | No |

Use `require_*` to guard state-changing operations. Use `check_*` only when you need a boolean (e.g. conditional UI / branching). A `check_*` alone does not block execution.

> Soroban limits contract export names to 32 bytes. Short names on the VE (`require_trustline_addrs`, `check_status_addrs`, …) cover the overloads — see the Validation Engine README.

#### `require_trustline!` / `require_trustline_addrs!` / `require_trustline_adv!`

Preferred macros: encode intent `data` from an action name + args, then call the VE.

```rust
require_trustline!(env, sender, amount, "transfer"());

require_trustline_addrs!(
    env, sender, amount,
    "transfer_to"(recipient),
    [recipient],
);

require_trustline_adv!(
    env, ValidationMode::Dapp, sender, amount,
    "transfer_to"(recipient),
    [recipient],
);
```

The action string and argument order must match what the Trustline backend / WebSDK sends as `functionPrototype` + positional `args`.

#### `require_trustline_raw`

**Enforcing call** (low-level). Requires an approved intent for `sender` / `value` / `data`. Panics if not compliant.

```rust
require_trustline_raw(&env, &sender, amount, &data);
```

#### `require_trustline_addrs_raw`

Same as above, plus an address list screened by policy / sanctions (e.g. recipient, token).

```rust
let addresses = soroban_sdk::vec![&env, recipient.clone(), token.clone()];
require_trustline_addrs_raw(&env, &sender, amount, &data, &addresses);
```

#### `require_trustline_adv_raw`

Same as `require_trustline_addrs_raw`, plus an explicit [`ValidationMode`].

```rust
require_trustline_adv_raw(
    &env, ValidationMode::Dapp, &sender, amount, &data, &addresses,
);
```

#### `check_trustline_status` / `check_status_addrs`

Non-destructive queries. Return `true` if compliant, `false` otherwise.

#### `set_validation_engine` / `validation_engine`

```rust
set_validation_engine(&env, &ve_address);
let ve = validation_engine(&env);
```

Stores the VE address under instance key `VE`.

#### `intent_id`

Deterministic intent id used by the oracle backend and the on-chain VE:

```text
sha256(xdr(network_id, mode, sender, protocol, value, data))
```

Backend and on-chain code **must** use the same function for reconciliation.

#### `encode_call_data` / `encode_intent`

Low-level helpers to build a stable `data` blob (prefer the `require_trustline*!` macros):

```rust
let data = encode_intent(&env, "pay", (destination, amount));
// or: encode_call_data(&env, "pay", &args_bytes);
```

### Types

- `ValidationMode` — currently `Dapp` only (reserved for future domains)

`TxState` lives in the **[stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine)** repository (oracle / backend surface), not in this SDK.

### Clients

- `ValidationEngineClient` — integrator CPI only (`require_trustline*` / `check_*`)

Oracle / backend methods (`add_tx`, `get_tx_state`, feature flags, …) use `ValidationOracleClient` from the [stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine) repository.

## Validation Modes

Modes only affect the intent hash domain. Policy content stays off-chain.

- **`Dapp`** (only supported mode for now) — Standard dapp validation

Additional modes may be introduced later. Use `require_trustline_adv!` / `require_trustline_adv_raw` when you need an explicit mode; today that must be `Dapp`.

## Examples

### Payment Forwarder

Guarded native / SEP-41 transfers. See [`contracts/payment-forwarder`](contracts/payment-forwarder):

- `pay_native` — SAC transfer (typically native XLM) guarded by Trustline
- `pay_tokens` — SEP-41 transfer guarded by Trustline

### Trustline Firewall (access-controlled gateway)

Protect a third-party contract **without** embedding Trustline in the target. See [`contracts/trustline-firewall`](contracts/trustline-firewall).

1. Deploy the target with **admin/owner = Trustline Firewall** address
2. Operate the target only through the firewall
3. Firewall runs Trustline, then CPI-forwards to the target

When a protocol has several distinct roles, deploy **one firewall per role** — each with its own operators and Trustline validation — rather than a single shared forwarder for all roles.

#### Access model

| Role | Scope |
|------|-------|
| **`owner`** | Single admin — `set_target`, `set_owner`, `set_operator`, `set_public_forward` (all Trustline-protected) |
| **`is_operator`** | Addresses allowed on `forward` when `public_forward` is false |
| **`public_forward`** | When `true`, any authenticated initiator may `forward` (still Trustline-gated) |

The owner is **not** an operator by default.

```rust
firewall.forward(
    &initiator,                        // business sender for Trustline + auth
    &Symbol::new(&env, "bump"),
    &Vec::new(&env),
);
```

| Concern | Behavior |
|---------|----------|
| Generic forward | `forward(initiator, fn_name, args)` → `env.invoke_contract` |
| Target caller | Target sees this firewall as the invoking contract |
| Trustline sender | `initiator` (explicit, like `pay_native(sender, …)`) |
| Operator gate | `is_operator` mapping, or `public_forward` for open access |
| Call payload | Explicit `Symbol` + `Vec<Val>` |

> Soroban has no catch-all fallback: `forward` is the idiomatic generic forwarder.

## Build

```sh
cargo test --workspace
stellar contract build
```

Requires the [Stellar CLI](https://developers.stellar.org/docs/tools/cli).

This repo is **self-contained**: runtime dependencies are `soroban-sdk` and the local `trustline-sdk` crate only. Example contracts (`payment-forwarder`, `trustline-firewall`) are templates — deploy a Validation Engine from [stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine) separately. End-to-end tests (oracle `add_tx` → guarded invoke) live in that repo.

## Publishing

This repository publishes the **`trustline-sdk`** crate to [crates.io](https://crates.io/crates/trustline-sdk). Example contracts (`payment-forwarder`, `trustline-firewall`, …) are **not** published — copy or fork them as templates.

Release history: [CHANGELOG.md](CHANGELOG.md).

## Related repositories

| Repo | Role |
|------|------|
| [stellar-sdk](https://github.com/TrustLine-id/stellar-sdk) | Repo: `trustline-sdk` crate + example contracts |
| [stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine) | On-chain VE, registry, sanctions |
| [stellar-demo-app](https://github.com/TrustLine-id/stellar-demo-app) | React demo (no Rust dependency) |

## Security Considerations

- Deploy the Validation Engine instance separately; configure its address in your `__constructor`
- Always `require_auth` on the business `sender` before calling Trustline helpers
- Include recipients / tokens in `require_trustline_addrs!` when they matter to policy or sanctions
- Keep `data` canonical and stable for a given protected method — it is part of the intent id
- Prefer upgrading the **VE instance** over forking logic into the SDK
- For `TrustlineFirewall`: set the firewall as the target's admin/owner; never leave a backdoor admin on the target that bypasses the firewall

## License

Copyright (c) 2026 [Trustline Digital Asset Ltd.](https://www.trustline.id). All rights reserved. MIT — see [LICENSE](LICENSE).

## Links

- **Homepage:** https://www.trustline.id
- **Repository:** https://github.com/TrustLine-id/stellar-sdk
- **Issues:** https://github.com/TrustLine-id/stellar-sdk/issues
- **Validation Engine:** https://github.com/TrustLine-id/stellar-validation-engine

## Support

Not sure how to get started? Contact us at contact@trustline.id
