# zixcel-github-transport

Execute typed GitHub requests while keeping credentials inside the configured custody boundary.

## What you can do

- Validate the admitted GitHub operation.
- Return bounded provider results through the declared transport.

## Current scope

The calling application supplies the operation grant and custody configuration. This package is not a general token-export interface.

Package distribution is not activated by this documentation. Use the owning product workspace and declared dependency versions. Registry availability is a separate release gate.

## Getting started

Install Rust 1.97 or newer. Run from the owning product workspace:

```sh
cargo test --locked -p zixcel-github-transport
```

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Examples](examples) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)

## Product responsibility

This component is maintained in [zixcel/zixcel-github](https://github.com/zixcel/zixcel-github). Use the [product README](../../README.md) for composition, use cases and trust boundaries.

## Ledger concurrency and recovery

Every reservation and completion locks the same persistent `<state_path>.lock`
sidecar while reading and atomically replacing the ledger. Keep both files in
an operator-controlled directory; do not delete or replace the sidecar while
processes use the ledger. OS locks are released when a process exits.

A prepared reservation left by a crash remains `provider-outcome-unknown`:
releasing the lock never authorizes another provider execution. Completed
receipts retain exact replay behavior. These guarantees require cooperating
processes to use the same state path and a filesystem supporting OS file locks.

`cargo test --locked -p zixcel-github-transport --test ledger` exercises concurrent
independent processes, preservation of distinct reservations/completions, and
recovery after killing a lock holder. It does not call GitHub.
