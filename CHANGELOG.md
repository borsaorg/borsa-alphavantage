# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

## [0.2.0]

### Breaking Change

- Temporarily disable earnings provider in `AvConnector`; `as_earnings_provider` returns `None`.

### Added

- `examples/showcase.rs` demonstrating quotes, search, equity history, and a simple forex request.

### Changed

- README updated with example usage, `.env` instructions, and dependency versions `0.2.0`.
- `.env` added to `.gitignore`.

### Dependencies

- Bump `borsa-core` to `0.2.0`.
- Update `paft` family to `0.6.0` via transitive dependencies.
- Enable `tokio` features: `macros`, `rt-multi-thread`.
- Add `dotenvy` as a dev-dependency for examples.

### Documentation

- Expanded README with steps to run the showcase example.
