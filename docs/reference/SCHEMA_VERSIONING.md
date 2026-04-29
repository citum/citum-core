# Schema Versioning Policy

This document defines how Citum versions the Rust workspace and the Citum schema,
and how those versions are now maintained by the release automation.

## Two-Track Versioning

Citum keeps code and schema versioning separate:

| Track | What | Source of truth | Automation |
|-------|------|-----------------|------------|
| **Code** | Rust workspace crates | `Cargo.toml` workspace version | manual `release-plz` workflow dispatch |
| **Schema** | Citum schema format exposed by `citum_schema::SCHEMA_VERSION` | `STYLE_SCHEMA_VERSION` in `crates/citum-schema-style/src/lib.rs` | PR-time validation + `scripts/bump.sh` |

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

1. A maintainer manually dispatches the workflow on `main`.
2. The workflow runs `release-plz` in `git_only` mode.
3. `release-plz` updates `Cargo.toml`, `Cargo.lock`, crate changelogs, and the root `CHANGELOG.md`.
4. `release-plz` opens or updates the release PR body directly from `.release-plz.toml`.

Do not use `scripts/bump.sh` for code versions or `v*` tags.

### Schema Releases

Schema versioning is decoupled from the code release workflow.

`scripts/validate-schema-release.py` does the following:

1. Regenerates `docs/schemas/*` in a temporary directory.
2. Fails if the generated schemas differ from the committed `docs/schemas/*`.
3. Detects whether the schema files or `STYLE_SCHEMA_VERSION` changed since the baseline ref.
4. Scans the commit range for exactly one valid `Schema-Bump:` footer when schema changes are present.
5. Verifies that the committed `STYLE_SCHEMA_VERSION` already matches the declared bump.

If schema files changed but no valid footer is present, CI fails before merge.

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
- Pull requests to `main` must commit the matching `STYLE_SCHEMA_VERSION` and `docs/schemas/*` updates in the same PR as the schema change.
- A rescue PR may carry one valid footer even when that PR does not itself change schema artifacts, as long as the merge is intended to unblock an existing schema-changing unreleased range on `main`.
- No footer is required when schema files and `STYLE_SCHEMA_VERSION` are unchanged.
- If a PR is squash-merged, preserve the footer in the squash commit body.
- The schema validation script treats generated-schema drift in `docs/schemas` as the canonical signal that the schema changed.

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

## Version Pinning Contract (Post-1.0)

Once Citum releases v1.0.0, schema consumers (IDEs, citum-hub, downstream tools) must
be able to pin to a specific schema version. This section defines how version pinning works.

### Pinning Formats

Consumers specify their required schema version in one of four ways:

| Format | Meaning | Example |
|--------|---------|---------|
| `"latest"` | Resolves to the highest published `schema-vX.Y.Z` tag at build/runtime. This is a distribution-layer concern; it does not appear in Rust schema code. | `schema: "latest"` |
| `"1"` | Major-version pin: matches any `1.x.y` schema. | `schema: "1"` |
| `"1.2"` | Minor-version pin: matches any `1.2.x` schema. | `schema: "1.2"` |
| `"1.2.3"` | Full semantic version pin: exact match only. | `schema: "1.2.3"` |

### Resolver Behavior

- **`"latest"`**: Always resolves to the highest published `schema-vX.Y.Z` tag. This is a tool/distribution decision, not a Citum runtime mechanism.
- **`"1"`**: Accepts `1.0.0`, `1.0.1`, `1.1.0`, `1.2.3`, etc. — any schema in the `1.x` line.
- **`"1.2"`**: Accepts `1.2.0`, `1.2.1`, `1.2.5`, but rejects `1.1.0` or `2.0.0`.
- **`"1.2.3"`**: Accepts only that exact version. Rejects `1.2.4` or `1.3.0`.

### Post-1.0 Bump Semantics

Starting from v1.0.0, the schema follows semantic versioning:

- **Patch** (e.g., `1.0.0` → `1.0.1`): Documentation fixes, validator improvements that do not change accepted data.
- **Minor** (e.g., `1.0.0` → `1.1.0`): New optional fields, new enum variants, additive changes. Existing styles continue to work; old tools can safely ignore new fields.
- **Major** (e.g., `1.0.0` → `2.0.0`): Required fields, removals, type changes. Existing styles may no longer validate; old tools must upgrade.

### Stability Recommendation

Style authors and tool builders who need predictable schema behavior should pin to a major version:

```
schema: "1"    # Stable: accepts any 1.x.y; will not jump to 2.x.y without author action
```

Using `"latest"` is suitable only for development or when tracking the absolute latest features.

### Implementation Note

The actual resolver logic (CLI flags, environment variables, API parameters) is deferred
to a follow-up task. This section defines the contract; runtime integration comes later.

## Manual Schema Bump Helper

`scripts/bump.sh` remains the single helper for changing `STYLE_SCHEMA_VERSION`.

Interactive usage:

```bash
./scripts/bump.sh schema patch
./scripts/bump.sh schema minor --dry-run
```

Automation usage:

```bash
./scripts/bump.sh schema minor --yes --no-validate
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
3. pull requests with schema changes commit the matching `STYLE_SCHEMA_VERSION`
4. `citum check` and auxiliary validation scripts read from the same canonical schema directory

## Schema Changelog

Track schema changes separately from code changes.

Historical note: entries below may predate the automation baseline and are the
authoritative record when matching tags were not created at the time.

#### schema-v0.38.0 (2026-04-29)
- Schema version bumped from 0.37.1 to 0.38.0
- Breaking: geographic place fields now use the transparent `Place` newtype while preserving string wire compatibility

#### schema-v0.37.1 (2026-04-28)
- Schema version bumped from 0.37.0 to 0.37.1
- Added `part_number`, `supplement_number`, `printing_number` shorthand fields to Monograph, Collection, CollectionComponent, SerialComponent, and Classic
- Added `NumberingType::Printing` for printing run identifiers
- Engine now resolves `PartNumber`, `SupplementNumber`, `PrintingNumber` variables via numbering values

#### schema-v0.37.0 (2026-04-26)
- Schema version bumped from 0.36.0 to 0.37.0
- Added `abstract_text` field to Monograph, CollectionComponent, and SerialComponent
- Djot inline markup now applies to Note and Abstract variables at render time

#### schema-v0.34.0 (2026-04-19)
- Schema version bumped from 0.33.0 to 0.34.0

#### schema-v0.33.0 (2026-04-18)
- Schema version bumped from 0.32.2 to 0.33.0

#### schema-v0.32.2 (2026-04-18)
- Schema version bumped from 0.32.1 to 0.32.2

#### schema-v0.32.1 (2026-04-17)
- Schema version bumped from 0.32.0 to 0.32.1

#### schema-v0.32.0 (2026-04-14)
- Add `grouped` field to `Citation` for cite-site dynamic compound grouping

#### schema-v0.31.0 (2026-04-14)
- Schema version bumped from 0.30.2 to 0.31.0

#### schema-v0.30.2 (2026-04-13)
- Schema version bumped from 0.30.1 to 0.30.2

#### schema-v0.30.0 (2026-04-12)
- Schema version bumped from 0.29.2 to 0.30.0 (major pre-1.0)
- Fixed style file consolidation to keep only embedded styles in `styles/embedded/`
- Restored HTML generation logic to compatibility report

#### schema-v0.29.2 (2026-04-11)
- Added `NameOrder::FamilyFirstOnly` variant (non-breaking additive)

#### schema-v0.29.1 (2026-04-10)
- Added `event: Option<WorkRelation>` to `Collection` to promote paper-conference
  event metadata (event-title, event-place) from note field to proper schema field

#### schema-v0.29.0 (2026-04-10)
- Schema version bumped from 0.28.0 to 0.29.0
- Added the template contributor role `chair` so event/session bibliography
  templates can render chair lists directly
- Clarified substitute-config merge semantics so bibliography or citation-local
  substitute presets do not discard top-level `role-substitute` chains

#### schema-v0.28.0 (2026-04-10)
- Schema version bumped from 0.27.1 to 0.28.0

#### schema-v0.27.1 (2026-04-09)
- Schema version bumped from 0.27.0 to 0.27.1
- Added `CollectionComponent.status`
- Updated the bib schema and generated bindings to expose the new field

#### schema-v0.27.0 (2026-04-08)
- Schema version bumped from 0.26.0 to 0.27.0
- Breaking: `Event` replaced separate `performer` and `organizer` fields with a
  unified `contributors: Vec<ContributorEntry>` list; update serialized Event
  payloads and generated bindings accordingly
- Added `available-date` to `Event`, `Monograph`, and `SerialComponent` to
  capture public-availability timing distinct from `issued`
- Added `series: Option<WorkRelation>` to `Event` for recurring event series
- Added `status`, `size`, `duration`, `references`, and `scale` to `Monograph`
- Added `section` and `status` to `SerialComponent`

#### schema-v0.26.0 (2026-04-07)
- Schema version bumped from 0.25.0 to 0.26.0
- Breaking: type refactor v3 unified contributor and publisher data shapes
- Added `created` as an origination date alongside `issued`

#### schema-v0.25.0 (2026-04-04)
- Schema version bumped from 0.24.0 to 0.25.0
- LegalCase.authority changed from required String to Option<String>

#### schema-v0.24.0 (2026-04-02)
- Schema version bumped from 0.23.0 to 0.24.0

#### schema-v0.23.0 (2026-04-01)
- Schema version bumped from 0.22.0 to 0.23.0

#### schema-v0.22.0 (2026-04-01)
- Schema version bumped from 0.21.0 to 0.22.0
- Breaking: split numbering semantics for generic `number`, `report`, and `part`
- Breaking: `NumberingType` adds `number` and `report`, and removes `book`
- Breaking: numbering accessors and rendering now preserve the narrowed generic/report boundary

#### schema-v0.21.0 (2026-04-01)
- Schema version bumped from 0.20.0 to 0.21.0
- Universal relational container model with recursive `WorkRelation`
- Controlled `Numbering` system for volume, issue, and edition
- Shorthand fields restored for better YAML ergonomics

#### schema-v0.19.0 (2026-03-31)
- Schema version bumped from 0.18.0 to 0.19.0

#### schema-v0.18.0 (2026-03-31)
- Schema version bumped from 0.17.1 to 0.18.0

#### schema-v0.17.0 (2026-03-30)
- Schema version bumped from 0.16.1 to 0.17.0

#### schema-v0.16.1 (2026-03-30)
- Schema version bumped from 0.16.0 to 0.16.1

#### schema-v0.15.2 (2026-03-29)
- Schema version bumped from 0.15.1 to 0.15.2

#### schema-v0.13.0 (2026-03-25)
- Schema version bumped from 0.12.0 to 0.13.0 (template-v2 additions)
- Note: pre-1.0 guard now ensures `Schema-Bump: major` increments minor, not major

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
