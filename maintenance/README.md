# GitHub repository maintenance

This owner-maintained Python package provides the recurring inventory and review
steps formerly written as workspace audit scripts. It uses Python 3.11+ and the
operator's existing authenticated `gh` and `git`; it never reads or saves tokens.
It is source-distributed here, independently of the published Rust crate. No
PyPI release is claimed. Install with `python3 -m pip install ./maintenance`, or
run from this repository with `PYTHONPATH=maintenance python3 -m
zixcel_github_maintenance`.

```sh
PYTHONPATH=maintenance python3 -m zixcel_github_maintenance inventory \
  --workspace /absolute/workspace --output /absolute/private-observation \
  --owner explicitly-managed-owner
PYTHONPATH=maintenance python3 -m zixcel_github_maintenance report \
  --inventory /absolute/private-observation
PYTHONPATH=maintenance python3 -m zixcel_github_maintenance pulls --patch \
  --inventory /absolute/private-observation --output /absolute/private-review
```

Inventory reads each accessible repository under the explicit owner list,
including private repositories. It checks default-branch protection, inherited
and local rules, open issues/PRs and branches, and local Git status. Missing or
denied observations are retained as such. `report` requires exact coverage of
the discovered repository IDs; a partial fleet never becomes complete. Output
is private operational evidence, not a new central task/credential authority.
Do not place the output inside a public source repository.

`pulls` records actual diffs, exact heads and all visible CI. `check-pull
--repository owner/repo --number 1 --expected-head <40-hex>` refreshes admission
immediately before an operator's normal GitHub merge. Drafts, conflicts, required
reviews, blocked merge states and absent/pending/failed CI are not eligible.
Eligibility does not review code, authorize a major-version migration, prove
native product acceptance or replace GitHub's enforced branch rules.

`apply --plan /absolute/private-plan.json --output /absolute/fresh-receipts`
applies an explicit `zix-github-maintenance-plan/v1` plan. Each operation includes
the repository ID and fresh-state expectations. Supported targets are narrowly
scoped metadata, absent active public-main rulesets and evidenced issue closure.
Visibility, billing, secrets, release tags and arbitrary API writes are excluded.
Existing protections are never weakened. Private-plan denials remain blockers.
Every write is preceded by a private receipt; a failed acknowledgment/readback
retains uncertainty and cannot be replayed using the same receipt directory.

`branches --inventory <observation> --workspace <root> --output <fresh-private-dir>`
fetches/prunes tracking refs and prepares merged-branch proofs. Add `--apply` for
explicit cleanup. Each remote deletion uses an exact-head lease and readback;
local deletion uses compare-and-swap. A branch must be an ancestor of the fetched
default branch or match the exact head of a merged same-repository/default-branch
PR whose merge commit is still in that history. Open PRs, protected branches,
dirty or other active worktrees and unproven commits are retained. Default
branches advance only by fast-forward. Nothing stashes, resets or cleans files.
Release tags and historical receipts are outside cleanup scope. A fresh output
directory is mandatory; stopped/uncertain operations remain visible.

Run the package boundary tests with:

```sh
PYTHONPATH=maintenance python3 -m unittest discover -s maintenance/tests -v
```

The separate `zix-repository-observer` remains the bounded read-only Rust
configuration observer. This operator utility does not change that contract.
Official publication-stage notice reconciliation stays with each owner's
existing `reconcile-npm-stages.yml`, including archive and Sigstore checks.
