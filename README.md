# zixcel-github

Build and validate typed GitHub requests for repository observation and explicitly enabled operations.

## What you can do

- Describe reads and supported repository changes.
- Validate request scope and parse provider results.

## Current scope

A caller supplies the authenticated transport and allowed operation configuration. Creation and deletion require explicit permission for the exact target.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Use the configured private registry when a dependency is not distributed publicly. Run from this repository:

```sh
cargo test --locked
```

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Examples](examples) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
