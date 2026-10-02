# zixcel-github interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

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
