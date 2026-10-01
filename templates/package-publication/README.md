# Public package publication templates

These files are standalone repository assets. Copy the appropriate workflow to
`.github/workflows/publish-npm.yml` or `.github/workflows/publish-cargo.yml`, and
copy `check-public-package.py` to `scripts/check-public-package.py`. A consumer
has no runtime dependency on the template repository.

## Package contracts

For npm, declare `repository.url` as `git+https://github.com/OWNER/REPOSITORY.git`,
`publishConfig.registry` as `https://registry.npmjs.org`, and `access` as `public`.
Use an explicit `files` allowlist and pinned `packageManager: pnpm@10.x.y` or
`npm@11.5.1` (the template default when packageManager is absent).
The template requires a committed lockfile, frozen installation, a test script,
a build script when compilation is needed, and
Node 24.15.0 with npm 11.5.1 or later. Build-generated public entry points must
be in the archive. Application repositories with `private: true` remain
applications and consume published libraries; do not remove that flag just to
make this workflow pass.

For Cargo, declare the public repository URL, `rust-version`, explicit package
version, license, and `publish = ["crates-io"]`. Public release dependencies
must resolve from crates.io. Remove private registries, source replacement,
and local or Git dependencies from the release checkout. This template covers
one independently released root crate. For a multi-crate repository, release
constituent crates in dependency order with explicit package selection and
separate per-crate versions; do not flatten a private workspace into a public
crate. Publish a provider and verify its registry version before regenerating
and testing consumer lockfiles.

## Establish the delivery route

1. Confirm the npm scope or global crate name and its owner. GitHub organization
   administration does not establish registry ownership. Complete the initial
   legitimate package registration if the registry requires it; do not publish
   an empty placeholder. The initial registration path must be reviewed using
   the exact tested and licensed package.
2. Configure a trusted publisher on the registry with the exact GitHub owner,
   repository, workflow filename, and environment. For npm direct publishing,
   explicitly permit `npm publish`; the current stage-only default is not
   sufficient. Do not add a long-lived publish token to these workflows.
3. Configure `npm-public` or `crates-io-public` as a protected GitHub environment
   with a required maintainer review. Protect `main`, require the existing test
   and security checks, and restrict release-tag creation. The workflow checks
   that the selected release tag points to a commit reachable from main. The
   default is `vVERSION`; set `REGISTRY_NPM_TAG_PREFIX=npm-v` and
   `REGISTRY_CARGO_TAG_PREFIX=crate-v` in repositories publishing both ecosystems.
   Multiple crates need separate package workflows and tag prefixes.
4. Complete full source and exact archive security, disclosure, license, and
   third-party notice review. Set environment variable
   `REGISTRY_SECURITY_APPROVED_SHA` to that reviewed commit SHA. Set repository
   variable `REGISTRY_PUBLISH_ENABLED=true` only after the identity and route
   are verified. Dispatch the workflow on the reviewed tag. New templates are
   disabled by default and no registry publish is caused by copying them.
5. Confirm the installed package from a fresh directory with no adjacent
   checkout, inherited private registry configuration, or local cache fallback.
   Check package identity, version, provenance where supported, and the consumer
   integration tests. Record source synchronization and package publication
   separately.

The archive gate rejects unsafe tar paths, links, private/generated files,
certificates, credential candidates, and missing licensing or package identity.
It is a bounded structural check, not a complete secrets or customer-data
scanner. Run the full security tool and registered disclosure checks separately
before approving the SHA. Retain prior applicable grants and third-party
notices. Install/build scripts execute only from the reviewed source revision.

Private packages use an authenticated private registry and a separate workflow;
these public templates never authorize their publication to public registries.
A public source repository alone does not mean its current package is ready to
publish. Version numbers are not changed by the templates.

## Official references

- https://docs.npmjs.com/trusted-publishers/
- https://docs.npmjs.com/generating-provenance-statements/
- https://doc.rust-lang.org/cargo/reference/publishing.html
- https://doc.rust-lang.org/cargo/reference/registries.html
- https://crates.io/docs/trusted-publishing
- https://github.com/rust-lang/crates-io-auth-action

Action commit pins were resolved from upstream release references on 2026-10-01.
Review updates to the templates and their pins before copying a new revision.
