# Native github observations

This Rust adapter uses the shared `zix-provider/v1` host and Crowsi
HTTPS/process boundaries. API identifiers, paths and server facts were checked
against the official source in `api/sources.lock.json`; `api/policy.json` separately
pins the route bytes, credentials, argument rules, identity checks and projections.
No upstream prose, examples or schema bodies are redistributed here.

Build with the existing Rust toolchain: `cargo build --locked -p zixcel-github-cli`.
The binary `zix-github` exposes `--manifest` without credentials. Register verified
binary/manifest hashes in Zix; the native runtime needs no Node installation.
Provide `ZIX_GITHUB_ACCESS_TOKEN` only in the selected process environment;
never put a bearer token in arguments, policy files, logs or Issue comments.
No implicit login, API activation, permission grant or credential fallback occurs.
The bounded HTTP transport sends its fixed package/version User-Agent, which
GitHub requires. Policy data cannot override that identifier or authorization.

Commands: `zix github user get`, `zix github repositories get`.
Use `zix github --help` for required scope arguments after installation.
This adapter only exposes reviewed reads, one response at a time. It does not
follow pagination or claim account-wide completeness, effective IAM or zero cost.
Responses include only declared scalar fields and validate selected resource IDs.

Existing owner TypeScript writes keep their separate Crowsi approvals. Importing
an upstream method does not enable it in this native adapter. Neither this source
change nor synthetic tests publish a package or prove real-account acceptance.
Native release/tag protection and distribution remain separate acceptance work.

Owner CI also prepares a static musl candidate with dependency notices, source
record and archive/contents hashes. CI artifacts are review candidates, not a
GitHub Release. The separate `release-native.yml` workflow admits version tags
`native/github/v*` from accepted main and publishes after the existing
`native-public` review. Native CLI versions are independent of the root Rust
library; a native HTTP fix does not require a new unrelated library version.
