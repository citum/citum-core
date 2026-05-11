# Release Workflow

This document describes how the Citum Core release pipeline works and what happens when code lands on `main`.

## Trigger

The release workflow is triggered when:
1. A non-release PR merges to `main` (automatic)
2. Manual workflow dispatch via GitHub Actions

The workflow does **not** trigger on direct pushes to `main` — all code arrives via merged pull requests.

## Conventional Commits and Bump Levels

The release workflow infers the bump level from conventional commit messages in the inspected commit range and applies the highest matching impact:

| Commit Type | Bump Level | Note |
|---|---|---|
| `feat:` | **Minor** | New feature release |
| `fix:`, `perf:` | **Patch** | Maintenance release |
| `feat!:`, `fix!:`, or footer `BREAKING CHANGE:` | **Major** | Breaking change (pre-1.0: capped at Minor) |
| `chore:`, `docs:` | None | No release triggered |

## Schema Version Bumps

If `crates/citum-schema-style/src/lib.rs` or `docs/schemas/` changes in the PR, the schema version is bumped **within the same bump level** as the workspace (e.g., if it's a patch release, schema bumps its patch version).

## Release PR Creation

When code merges to `main`, the `detect` job:
1. Reads the commit history since the last tag
2. Infers the bump level
3. Sends output to the `release-pr` job

The `release-pr` job then:
1. Creates a branch named `release/next`
2. Bumps all crate versions via `cargo-release`
3. Updates schema versions and regenerates schema files
4. Creates a pull request against `main` titled `chore: release v<version>`
5. Updates that PR on subsequent release-triggering merges until it is merged

All release levels (`patch`, `minor`, pre-1.0-capped `major`) use this same `release/next` PR flow.

## Tag and Publish (Future)

When the release PR merges to `main`, the `auto-tag` job:
1. Detects the merged release PR from `release/next`
2. Tags the workspace version as `v<version>`
3. Tags the schema version as `schema-v<schema-version>` (if schema changed)
4. Pushes both tags to the remote

**Publishing is deferred.** When ready to publish crates to crates.io:

```bash
# Publish only public crates (in dependency order)
cargo publish -p citum-edtf
cargo publish -p citum-schema-data
cargo publish -p citum-schema-style
cargo publish -p citum-schema
cargo publish -p citum-migrate
cargo publish -p citum-engine

# Schema is not published to crates.io — it's versioned via git tags
```

## Repository Settings

To enable the release pipeline, ensure:

1. **Branch Protection on `main`**:
   - Require status checks before merge
   - Require at least 1 approving review (if enforced by your team)

2. **Auto-Merge Settings**:
   - Enabled in repo settings
   - Allow squash merge (preferred for release PRs)

3. **Secrets** (for future publishing):
   - Add `CARGO_REGISTRY_TOKEN` to repo secrets
   - Token should have publish scopes for the crates.io registry
   - Only enable publishing step when ready (currently disabled)

## Crate Visibility

### Public Library Crates (Published to crates.io)

These crates are published when the release workflow runs:
- `citum-schema`
- `citum-schema-data`
- `citum-schema-style`
- `citum-migrate`
- `citum-edtf`
- `citum-engine`

Each has a descriptive `description` field in its `Cargo.toml` and inherits workspace metadata (`repository`, `homepage`, `authors`).

### Internal Crates (Not Published)

These crates are marked `publish = false` in their `Cargo.toml`:
- `csl-legacy` — CSL 1.0 XML parser (internal use only)
- `citum` (citum-cli) — CLI binary (distributed separately)
- `citum-server` — JSON-RPC server (distributed separately)
- `citum-analyze` — Analysis and testing tools (internal use only)
- `citum-pdf` — Typst PDF rendering (internal; may be published later)
- `citum_store` — Configuration/cache storage (internal; may be published later)
- `citum-bindings` — Language bindings (internal; published separately as language-specific packages)

## Troubleshooting

**Release PR fails to create:**
1. Check that the merged PR is not from `release/next`
2. Verify the commit message follows conventional commit format
3. Check workflow logs under `.github/workflows/release.yml` for details

**Release PR is not created or updated:**
1. Verify the release workflow has `pull-requests: write` and `contents: write`
2. Confirm `RELEASE_TOKEN` is available and has repo write permissions
3. Check whether the release trigger inferred `should-release=false`

**Tag not created after PR merges:**
1. Ensure the `auto-tag` job has `contents:write` permission
2. Check that the release PR was merged from `release/next` branch

## See Also

- `CLAUDE.md` — Citum project instructions document the versioning signals
- `release.toml` — cargo-release configuration
- `scripts/infer-release-bump.py` — bump inference logic
