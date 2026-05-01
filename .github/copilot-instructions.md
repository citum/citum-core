# Copilot Review Instructions

## Workspace versioning

Do **not** flag workspace version bumps in feature or fix PRs as a workflow violation.

This repo uses a pre-commit hook (`scripts/rust-check.py`) that automatically bumps the `[workspace.package].version` field in `Cargo.toml` whenever a publishable Rust crate is modified. The hook also adds a `Version-Bump: patch|minor|major` footer to the commit and stages the updated `Cargo.toml` and `Cargo.lock`. This is intentional: the version on the branch reflects the semver impact of that branch's changes.

`release-plz` reads this version on merge and uses it to drive the release PR and changelog. The two mechanisms are designed to work together — release-plz does not re-calculate semver; it publishes the version the hook committed.

**Consequence:** Every PR that modifies a Rust source file will contain a workspace version bump. This is correct behaviour, not a workflow violation.

## Bean files

`.beans/` contains task-tracking files managed by the `beans` CLI (an agentic issue tracker). These files are intentionally committed alongside code changes. Do not flag them as unrelated or suggest removing them from commits.

Archived beans move to `.beans/archive/` when completed. Both paths are expected in the repository.

## Commit footer conventions

Commits may include non-standard footers alongside the conventional `Co-Authored-By` line:

- `Version-Bump: patch|minor|major` — set by the pre-commit hook; documents semver impact
- `Schema-Bump: patch|minor|major` — set when `docs/schemas/` is regenerated
- `Refs: <bean-id>` — links the commit to a bean tracking entry
