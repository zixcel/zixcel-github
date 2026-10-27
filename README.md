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


## GitHub integration workspace

This repository owns the GitHub API wrapper, provider-specific egress contracts
and credential-bound GitHub execution adapter under `crates/`.
The generic HTTP transport does not decide which GitHub operations are allowed.

```sh
cargo test --locked --workspace --all-targets
cargo test --locked -p zixcel-github-transport --test contracts
```

These cases validate request binding, provider contracts and replay boundaries
using fixtures. Live reads and repository mutations require an explicit caller
authorization and deployment-owned credential configuration. OpenAI authorization
egress belongs to `zixcel-openai-auth`, not this GitHub workspace.

[OSS responsibility and source completion](docs/oss-source-contract.md) defines the supported package boundary and required acceptance. It does not declare unexecuted source, distribution or production checks complete.

Dependency updates: [repository policy](docs/dependency-updates.md).
Registry releases: [version tags and GitHub OIDC](docs/registry-release.md).

## Native Zix provider

[Reviewed Rust adapter](crates/github-cli/README.md) adds scoped read commands
through the common `zix` protocol. Source/development tests and native distribution
are separate; no real credentials or cloud operations are used by CI.

Native Zix operation discovery and diagnostic handling: [operation guidance](docs/operation-guidance.md).
