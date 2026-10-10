# zixcel-github

Typed GitHub API request planning, validation and receipt parsing with an injected execution backend. Use this library to keep repository observation and explicitly allowed changes behind a reviewable provider boundary.

## Install

```toml
[dependencies]
zixcel-github = "0.1.1"
```

This dependency uses crates.io; no private registry is required.

## Capabilities

- Parse connector configuration and build capability-aware plans.
- Validate supported API requests and target expectations.
- Parse snapshot, text and workflow-input artifacts into typed results.

## Example

```rust
use zixcel_github::{CONNECTOR, PROVIDER};

assert_eq!(CONNECTOR, "zixcel-github");
assert_eq!(PROVIDER, "github");
```

## Features and requirements

Requires Rust 1.97 or newer. The crate is a reusable library, not a command-line application.

## Boundaries

The caller supplies authentication and the transport backend. Creation, deletion and other changes require explicit target permissions. This crate does not store tokens or choose application policy.

## Development

```sh
cargo fmt --all -- --check
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Documentation and license

[API documentation](https://docs.rs/zixcel-github) · [Source](https://github.com/zixcel/zixcel-github) · [Usage guide](https://github.com/zixcel/zixcel-github/blob/main/docs/getting-started.md)

Apache-2.0. Retain the package LICENSE and NOTICE; see the source repository for security reporting and contribution guidelines.
