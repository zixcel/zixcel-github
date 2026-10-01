# zixcel-github

A typed GitHub API wrapper for normalized observations, a closed set of operations and backend invocation through opaque connection references. Public contracts contain no caller-specific packages, runtime grants, workflows or user authorization concepts.

```bash
cargo run --offline -- doctor
cargo run --offline -- capabilities
cargo run --offline -- validate examples/config.toml
cargo run --offline -- plan examples/config.toml
cargo run --offline -- check-request examples/push-request.json
```

Only opaque `connection_ref` values are accepted, never credentials. OAuth, secret resolution and HTTP/Git transport belong to Crowsi/Zixcel worker boundaries. `execute_api_request` passes validated closed requests to an injected `GitHubBackend`; higher adapters authorize the caller. No path dependencies on other local repositories are used.

`parse_config` validates closed TOML up to 1 MiB; `build_plan` is independent of repository ordering. No HTTP or secret resolver is linked; execution is restricted to an injected backend trait. The source manifest now targets crates.io. The public publication workflow is prepared locally and remains disabled pending name ownership, trusted-publisher setup, and exact package review. Existing private-registry artifacts have not been replaced or activated.

## Responsibilities

- Zixcel: API requests/receipts, operation validation and backend ports.
- Crowsi: connection resolution, credentials, transport and signed invocation boundaries.
- Caller adapters: conversion of user/workflow authority into API requests.

The crate does not identify adapter types or products. Write operations include private repository creation, source snapshots, issues/PRs, workflow dispatch and private repository deletion.

## Quality gate

These checks are local and do not connect to GitHub:

```bash
# WONDERLAND_ROOT is the workspace checkout root.
"$WONDERLAND_ROOT/bin/verify-repositories" --rust --tier standard
```

## Local policy and synchronization

`allowed_actions` is the local write ceiling. Missing or empty lists allow reads only. `ConnectorConfig::authorize_request` matches connection, organization and repository. Signed upstream grants and GitHub permissions remain separate requirements.

```sh
cargo run -- check-configured-request examples/write-config.toml examples/create-request.json
```

`execute_configured_request` validates configuration before invoking the backend. Production Crowsi transport applies the same checks in verify and execute. `execute_api_request` is a lower-level typed port; callers must apply authorization and local ceilings.

`create-private-repository`, `push-repository-snapshot` and `delete-repository` are independent permissions. Deletion requires `github-repository-administration/delete-resource`, an expected repository ID and a backup-declaration SHA-256.

`zixcel-repository-security prepare-snapshot` prepares a checked complete-source artifact. References use `sha256:<64hex>`; digest fields use bare 64-character hex. Store `<64hex>.json` in Crowsi's artifact store. Hash the current remote commit SHA string using SHA-256 and use the bare digest in `expected_remote: {state: exact, commit_sha256: ...}`. Reobserve after changes. Force updates are never used.

Creation distinguishes users and organizations; organizations use `/orgs/{owner}/repos`. Because Git Data API cannot operate on an empty repository, creation initializes a commit; observe its head before pushing a complete snapshot. Creation and synchronization are separate signed operations. Snapshots do not transfer local Git history. History-preserving pushes require Crowsi Git transport and separate history inspection.

## Deletion backup declaration

```json
{"schema":"zixcel://github/deletion-backup/v1","repository_id":"12345","owner":"example-org","repository":"example-api","bundle_digest_sha256":"sha256:<64hex>"}
```

The declaration's digest and target are verified, but the declaration alone does not prove backup existence or completeness. Operators granting deletion must separately verify and retain Git bundles, LFS, issues, settings and other required backups.

Validation used local fixtures. Live Crowsi enrollment, authority adoption and create/push/delete acceptance tests remain unperformed. Publishing this repository does not activate these runtime operations.

## License

Apache-2.0; see LICENSE and NOTICE. Previously granted permissions and third-party terms remain effective. Private registration, credentials and runtime state are excluded. Generated `.tgz` archives are excluded from source and distribution.

## Package integration

The package is an independently consumable unit. Callers reference its documented
interface through a versioned dependency and own application-specific composition
and integration.

## Package publication templates

`templates/package-publication/` contains standalone npm and crates.io GitHub Actions templates, a public-package and archive gate, and activation policy. They are copied into each consuming repository and do not add a runtime dependency on this crate. Publishing is disabled until the registry identity, protected environment, reviewed source SHA, and package delivery prerequisites are configured. See the template policy and installation instructions in that directory.
