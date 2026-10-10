# Using zixcel-github-egress-contracts

Declare exactly where a provider request may go and how much data it may transfer.

## Before you start

The contracts describe allowed communication; the caller must supply authorization and enforcement.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Validate destination, method and size limits.
- Share a strict egress contract with transport adapters.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)
