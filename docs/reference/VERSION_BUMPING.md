# Workspace Version Bumping Policy

This document describes how the Citum workspace version (`[workspace.package].version`
in root `Cargo.toml`) is managed.

## Two-Track Versioning

Citum keeps code and schema versioning separate:

| Track | What | Source of truth | Automation |
|-------|------|-----------------|------------|
| **Code** | Rust workspace crates | `[workspace.package].version` in `Cargo.toml` | `cargo-release` via GitHub Actions release workflow |
| **Schema** | Citum schema format | `STYLE_SCHEMA_VERSION` | automated release workflow |

## How It Works

### Automated release workflow (`.github/workflows/release.yml`)

When code merges to `main`, the release workflow:

1. **Detects which tracks changed**:
   - Structural changes to committed `docs/schemas/*.json` artifacts → schema track
   - Rust workspace crate, `Cargo.toml`, or `Cargo.lock` changes → code track
2. **Infers bump level** from conventional commit messages since the last tag:
   - `fix:`, `perf:` → patch
   - `feat:` → minor
   - `feat!:` or `BREAKING CHANGE:` → major (capped at minor for pre-1.0)
3. **Runs `cargo-semver-checks`** as a safety net — if it detects breaking API changes
   but the commits only say `fix:`, the level is escalated.
4. **Opens or updates a release PR** on `release/next` (for all release levels):
    - Runs `cargo release <level> --workspace` to bump `Cargo.toml`
    - Runs `git-cliff` to generate the root `CHANGELOG.md`
    - If generated schema artifacts changed structurally, also bumps `STYLE_SCHEMA_VERSION`
5. **Auto-tags** when the release PR is merged to `main`.

### No Version-Bump footers

Workspace code versioning no longer uses commit footers. Do not add `Version-Bump:`
footers to commits. The release workflow infers the bump level automatically.

## Publishable Crates

These crates contribute to the release workflow's path detection:

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

Before 1.0, `major` bumps are folded into `minor` to stay in the `0.x.y` range.
A 1.0 promotion requires a manual release.

## Configuration

The release tool is configured in `release.toml`:
- `shared-version = true` — all workspace crates share one version
- `consolidate-commits = true` — one version-bump commit per release
- `dependent-version = "upgrade"` — intra-workspace deps are auto-updated
- root `CHANGELOG.md` is the single workspace changelog; crate-local changelogs
  are intentionally not tracked

## Related

- [SCHEMA_VERSIONING.md](SCHEMA_VERSIONING.md) — schema track
- `.github/workflows/release.yml` — release workflow
- `release.toml` — cargo-release configuration
