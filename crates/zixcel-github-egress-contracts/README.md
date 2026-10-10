# zixcel-github-egress-contracts

Validate bounded GitHub authorization, operation bindings and correlated receipts.
This library never stores credentials or calls the GitHub API.

## What you can do

- Validate destination, method and size limits.
- Share a strict egress contract with transport adapters.

## Current scope

The contracts describe allowed communication; the caller must supply authorization and enforcement.

This reusable library is packaged independently for crates.io. Add `zixcel-github-egress-contracts = "0.10.0"` to a Rust 1.97 application. Its dependencies use the public registry; a sibling checkout or private registry is not required.

## Getting started

Install Rust 1.97 or newer. Run from the owning product workspace:

```sh
cargo test --locked -p zixcel-github-egress-contracts
```

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Implementation and public interfaces](src) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)

## Product responsibility

This component is maintained in [zixcel/zixcel-github](https://github.com/zixcel/zixcel-github). Use the [product README](../../README.md) for composition, use cases and trust boundaries.

## Use case: bind an opaque request document

```rust
use zixcel_github_egress_contracts::github_document_digest;

let exact_request_json = r#"{"operation":"read-only-preview"}"#;
let digest = github_document_digest(exact_request_json);
assert!(digest.starts_with("sha256:"));
```

Hash the exact serialized bytes embedded in a command. The command validator
checks every document digest, identity and finite authorization window; receipt
validators correlate the verification and execution results with that command.
A content digest alone is not authorization. The caller separately verifies
the issuer signature and owns credential custody and transport enforcement.
