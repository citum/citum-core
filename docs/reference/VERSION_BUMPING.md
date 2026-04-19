# Workspace Version Bumping Policy

This document describes how the Citum workspace version (`[workspace.package].version`
in root `Cargo.toml`) is detected, bumped, and validated across commits.

## Two-Track Versioning

Citum keeps code and schema versioning separate:

| Track | What | Source of truth | Automation |
|-------|------|-----------------|------------|
| **Code** | Rust workspace crates | `[workspace.package].version` in `Cargo.toml` | pre-commit auto-bump + `Version-Bump:` footer |
| **Schema** | Citum schema format | `STYLE_SCHEMA_VERSION` in `crates/citum-schema-style/src/lib.rs` | pre-commit auto-bump + `Schema-Bump:` footer |

## How It Works

### pre-commit hook

When you stage `.rs` files in a publishable crate, `scripts/rust-check.py pre-commit` runs
automatically and:

1. Tries `cargo-semver-checks --baseline-rev HEAD --workspace` to detect API-breaking changes.
2. Falls back to a diff heuristic if `cargo-semver-checks` is not installed:
   - Removed `pub` items → `major`
   - Added `pub` items → `minor`
   - No public surface change → `patch`
3. Bumps `[workspace.package].version` in `Cargo.toml` and stages it.
4. Also updates `Cargo.lock` if the `cargo update` invocation succeeds.
5. Writes a `.git/VERSION_BUMP` handoff file with the inferred bump level.

### commit-msg hook

`scripts/rust-check.py commit-msg` reads the handoff file and appends a footer to the
commit message:

```
Version-Bump: patch
```

Valid values: `patch`, `minor`, `major`.

### pre-push hook

`scripts/validate-version-release.py` validates the full push range:

- Collects all `Version-Bump:` footers from commits in the range.
- Resolves the highest bump level (`major > minor > patch`).
- Verifies that `Cargo.toml` version at HEAD equals `bump_version(baseline, level)`.

### CI (PR check)

The `checks` job runs `validate-version-release.py` on the PR commit range.

The `semver-check` job runs `cargo-semver-checks-action` for a definitive API surface
comparison, posting results as a PR check.

## Publishable Crates

These crates contribute to the `Version-Bump` contract:

- `crates/csl-legacy`
- `crates/citum-schema-data`
- `crates/citum-schema-style`
- `crates/citum-schema`
- `crates/citum-migrate`
- `crates/citum-engine`
- `crates/citum-cli`
- `crates/citum-bindings`

Excluded (internal tooling, experimental, or not yet published):

- `crates/citum-analyze`
- `crates/citum-pdf`
- `crates/citum-server`
- `crates/citum_store`
- `crates/citum-edtf`

## Bump Level Rules

| Situation | Level |
|-----------|-------|
| Public item removed or signature changed | `major` (pre-1.0: treated as `minor`) |
| New public item added | `minor` |
| Bug fix, internal refactor, test, docs | `patch` |

Before 1.0, `major` bumps are folded into `minor` to stay in the `0.x.y` range, matching
release-plz behaviour. A 1.0 promotion requires a manual release.

## Override / Rescue

If the auto-bump inferred the wrong level, override manually:

1. Edit `Cargo.toml` `[workspace.package].version` to the correct value.
2. Stage `Cargo.toml` (and `Cargo.lock` if it changed).
3. Add `Version-Bump: <correct-level>` to the commit message body.

The commit-msg hook validates that the staged version matches the declared footer.

## Skipping

Set `SKIP_VERSION_CHECK=1` to bypass the pre-commit and commit-msg checks for a single
commit (use sparingly — CI will still validate on PRs).

## Related

- [SCHEMA_VERSIONING.md](SCHEMA_VERSIONING.md) — schema track
- `scripts/rust-check.py` — pre-commit and commit-msg implementation
- `scripts/validate-version-release.py` — pre-push and CI validation
