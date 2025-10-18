# borsa-alphavantage

[![Crates.io](https://img.shields.io/crates/v/borsa-alphavantage)](https://crates.io/crates/borsa-alphavantage)
[![Docs.rs](https://docs.rs/borsa-alphavantage/badge.svg)](https://docs.rs/borsa-alphavantage)
[![Downloads](https://img.shields.io/crates/d/borsa-alphavantage)](https://crates.io/crates/borsa-alphavantage)
[![License](https://img.shields.io/crates/l/borsa-alphavantage)](LICENSE)

Alpha Vantage connector for the borsa financial data ecosystem.

## Overview
`borsa-alphavantage` implements the `BorsaConnector` trait using the Alpha Vantage API to provide quotes, historical data, and fundamentals.

## Installation
```toml
[dependencies]
borsa-alphavantage = "0.2.0-alpha.1"
borsa-core = "0.2.0-alpha.1"
```

## Usage

Refer to the main `borsa` crate for how to register connectors. This crate implements `BorsaConnector` and can be added to a `borsa` client builder.

> **Feature flag:** Closure-based adapter helpers (for dependency-free tests) live behind the
> optional `test-adapters` feature. Enable it in `Cargo.toml` or via
> `cargo test --features borsa-alphavantage/test-adapters` whenever you depend on the mocks.

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md). Please also read our [Code of Conduct](CODE_OF_CONDUCT.md).

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
