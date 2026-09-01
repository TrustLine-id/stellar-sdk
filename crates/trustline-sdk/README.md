# trustline-sdk

A Rust / Soroban SDK for protecting Stellar smart contracts from unauthorized access and malicious transactions by integrating Trustline's Oracle with on-chain data sources.

**Compatible with `soroban-sdk` 27.x.**

## 30-second example

```toml
[dependencies]
trustline-sdk = "0.1"
soroban-sdk = "27"
```

```rust
use soroban_sdk::{Address, Bytes, Env};
use trustline_sdk::{encode_call_data, require_trustline, set_validation_engine};

pub fn __constructor(env: Env, validation_engine: Address) {
    set_validation_engine(&env, &validation_engine);
}

pub fn transfer(env: Env, sender: Address, amount: i128) {
    sender.require_auth();
    let data = encode_call_data(&env, "transfer", &Bytes::new(&env));
    require_trustline(&env, &sender, amount, &data);
}
```

Deploy a Validation Engine instance separately ([stellar-validation-engine](https://github.com/TrustLine-id/stellar-validation-engine)), then pass its contract id to `set_validation_engine`.

## License

Copyright (c) 2026 [Trustline Digital Asset Ltd.](https://www.trustline.id). All rights reserved. MIT — see [LICENSE](../../LICENSE) in the repository root.

## More documentation

- [Repository README](https://github.com/TrustLine-id/stellar-sdk#readme) — architecture, API reference, example contracts
- [Changelog](https://github.com/TrustLine-id/stellar-sdk/blob/master/CHANGELOG.md)
