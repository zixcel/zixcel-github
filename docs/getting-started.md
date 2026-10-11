# Using zixcel-github

Build and validate typed GitHub requests for repository observation and explicitly enabled operations.

## Before you start

A caller supplies the authenticated transport and allowed operation configuration. Creation and deletion require explicit permission for the exact target.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Describe reads and supported repository changes.
- Validate request scope and parse provider results.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)

The native GitHub transport requires curl 8.4 or newer to enforce `--max-filesize` during transfers of unknown length. Bodies are limited to 2 MiB during transfer and reads; response headers use a pipe with a 64 KiB limit. Oversized headers terminate and reap the child. Temporary exchange files are removed on failure.
