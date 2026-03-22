# Schema Versioning Policy

This document defines how Citum versions the Rust workspace and the Citum schema,
and how those versions are now maintained by the release automation.

## Two-Track Versioning

Citum keeps code and schema versioning separate:

| Track | What | Source of truth | Automation |
|-------|------|-----------------|------------|
| **Code** | Rust workspace crates | `Cargo.toml` workspace version | `release-plz` |
| **Schema** | Citum schema format exposed by `citum_schema::SCHEMA_VERSION` | `STYLE_SCHEMA_VERSION` in `crates/citum-schema-style/src/lib.rs` | release-prep + `scripts/bump.sh` |

Why the split:

- Code refactors, performance work, and new APIs do not force schema bumps.
- Schema changes bump only when the wire format or generated schemas actually change.
- Users can reason about compatibility separately from processor release cadence.

## Sources Of Truth

- Code release notes: root [`CHANGELOG.md`](../../CHANGELOG.md)
- Code version tags: `vX.Y.Z`
- Schema version constant: `crates/citum-schema-style/src/lib.rs`
- Canonical committed JSON schemas: `docs/schemas/*.json`
- Operational schema history: this file
- Future normative compatibility docs: bean `csl26-fuw7`

The root changelog is workspace-wide. The `citum` package owns it in
`.release-plz.toml`, but its changelog now includes the other synchronized
workspace crates so the top-level release PR reflects the full release.

## Automated Release Flow

### Code Releases

Code releases are prepared by `.github/workflows/release-plz.yml`.

1. The workflow looks up the latest root `v*` tag.
2. `scripts/prepare-release-artifacts.py` runs before `release-plz release-pr`.
3. `release-plz` updates `Cargo.toml`, `Cargo.lock`, crate changelogs, and the root `CHANGELOG.md`.
4. The workflow rewrites the release PR body so it explicitly calls out the current schema version and whether it changed.

Do not use `scripts/bump.sh` for code versions or `v*` tags.

### Schema Releases

Schema release prep now happens in the same workflow that prepares the code release PR.

`scripts/prepare-release-artifacts.py` does the following:

1. Regenerates `docs/schemas/*` with `cargo run --bin citum --features schema -- schema --out-dir docs/schemas`.
2. Detects whether the generated schema files changed, or whether `STYLE_SCHEMA_VERSION` already changed since the last root `v*` tag.
3. Scans unreleased commits for exactly one `Schema-Bump:` footer.
4. If schema files changed and the schema version did not already move, runs `./scripts/bump.sh schema <patch|minor|major> --yes --no-validate --no-commit --no-tag`.
5. Regenerates `docs/schemas/*` again after the version bump so the committed JSON schemas match the new default schema version.

If schema files changed but no valid footer is present, the release workflow fails before opening or updating the release PR.

Pull requests to `main` now run the same validation logic against the PR base
commit in dry-run mode. That means schema-affecting PRs fail in CI before merge
if they do not carry exactly one valid footer in the PR commit range.
The PR check also allows a single rescue footer on a PR that does not itself
change schema artifacts, so a follow-up PR can unblock an already broken
release range on `main`.

## Schema Bump Contract

The only supported schema bump marker is a commit footer:

```text
Schema-Bump: patch
Schema-Bump: minor
Schema-Bump: major
```

Rules:

- Exactly one `Schema-Bump:` footer must appear across the unreleased commit range when schema changes are present.
- Pull requests to `main` use the same rule against the PR commit range (`base..HEAD`), so the footer is enforced before merge as well as at release time.
- A rescue PR may carry one valid footer even when that PR does not itself change schema artifacts, as long as the merge is intended to unblock an existing schema-changing unreleased range on `main`.
- No footer is required when schema files and `STYLE_SCHEMA_VERSION` are unchanged.
- If a PR is squash-merged, preserve the footer in the squash commit body.
- The release prep script treats generated-schema drift in `docs/schemas` as the canonical signal that the schema changed.

### Choosing Patch, Minor, Or Major

**Patch**

- Documentation clarifications in generated schema metadata
- Validator fixes that do not change the accepted data model
- Non-structural schema metadata corrections

**Minor**

- New optional fields
- New non-breaking enum variants
- New preset or registry structures that extend the format
- Backward-compatible additions to generated JSON schemas

**Major**

- Required field additions
- Field removals or renamed fields without compatibility shims
- Type changes that invalidate existing documents
- Semantic changes that require style authors to rewrite existing data

## Tags And Baseline

Schema tags continue to use the `schema-vX.Y.Z` prefix.

Historical schema tags before this automation are incomplete. At the time this
policy was updated, the repo retained `schema-v0.7.1`, but later schema history
was tracked primarily in documentation rather than a complete tag chain. Treat
that earlier period as pre-automation legacy.

The first post-automation schema bump establishes the new automation baseline.
From that point forward:

- schema bumps must be driven by the `Schema-Bump:` footer contract
- `STYLE_SCHEMA_VERSION`, `docs/schemas/*`, and the schema changelog entry in this file move together
- future schema tags should continue from the automation-produced version line

## Manual Schema Bump Helper

`scripts/bump.sh` remains the single helper for changing `STYLE_SCHEMA_VERSION`.

Interactive usage:

```bash
./scripts/bump.sh schema patch
./scripts/bump.sh schema minor --dry-run
```

Automation usage:

```bash
./scripts/bump.sh schema minor --yes --no-validate --no-commit --no-tag
```

The helper updates:

1. `STYLE_SCHEMA_VERSION`
2. the schema changelog section in this file
3. optional validation / commit / tag actions, depending on flags

## CI Validation

The repo now treats `docs/schemas` as the only committed schema artifact set.

CI validates:

1. all schema-generating code still produces the checked-in `docs/schemas/*`
2. pull requests with schema changes include exactly one valid `Schema-Bump:` footer in their commit range
3. the release-prep step can derive a valid schema bump decision from release-range commit metadata
4. `citum check` and auxiliary validation scripts read from the same canonical schema directory

## Schema Changelog

Track schema changes separately from code changes.

Historical note: entries below may predate the automation baseline and are the
authoritative record when matching tags were not created at the time.

#### schema-v0.12.0 (2026-03-22)
- Schema version bumped from 0.11.0 to 0.12.0

#### schema-v0.11.0 (2026-03-22)
- Schema version bumped from 0.10.0 to 0.11.0

#### schema-v0.10.0 (2026-03-19)
- Schema version bumped from 0.9.0 to 0.10.0
- Added style-level preset support and null-aware preset overlay semantics

#### schema-v0.8.0 (2026-03-05)
- Schema version bumped from 0.7.1 to 0.8.0
- Breaking change: compound grouping moved from per-reference `group-key`
  to top-level bibliography `sets`

#### schema-v0.7.1 (2026-03-02)
- Schema version bumped from 0.7.0 to 0.7.1

## Follow-Up Work

- Bean `csl26-fuw7` owns the deeper compatibility contract docs:
  `docs/architecture/design/VERSIONING.md` and `docs/architecture/SCHEMA_CHANGELOG.md`
- Bean `csl26-yipx` still owns runtime enforcement of `Style.version` in `citum check`

## References

- [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [release-plz Documentation](https://release-plz.ieni.dev/docs/config)
