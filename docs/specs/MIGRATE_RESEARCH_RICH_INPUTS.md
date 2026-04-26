# Migrate-Research Rich-Input Pass Contract

**Status:** Active
**Date:** 2026-04-02
**Related:** [FIDELITY_RICH_INPUTS.md](./FIDELITY_RICH_INPUTS.md), [DIVERGENCE_REGISTER.md](../adjudication/DIVERGENCE_REGISTER.md)

## Purpose
Define the bounded rich-input workflow for `migrate-research` when a style-fidelity pass needs supplemental benchmark evidence without falling back to full-corpus exploration as the first debugging surface.

## Pass Contract
Each pass must target exactly one bounded cluster.

Required pass fields:

- target style
- target cluster
- cluster selector
- semantic class
- implementation form
- selected parent, if any
- primary oracle before and after
- official supplemental before and after
- cluster before and after
- classification
- hypothesis
- stop reason

## Evidence Ladder
Use evidence in this order:

1. Primary oracle hard gate
2. Official style-scoped report from `report-core`
3. Cluster-scoped supplemental evidence from a reduced rich-input fixture
4. Full supplemental benchmark rerun as confirmation

The full supplemental corpus is confirmation evidence, not the first debugging surface.

## Classification
Every pass must classify the chosen cluster before edits:

- `style-defect`
- `migration-artifact`
- `processor-defect`
- `intentional divergence`

Every pass must also classify the target style on the taxonomy axes before edits:

- semantic class: `base`, `profile`, `journal`, `independent`, or `unknown`
- implementation form: `alias`, `config-wrapper`, `structural-wrapper`, `standalone`, or `unknown`

Use `unknown` when the target cannot yet be resolved safely on that taxonomy
axis; do not force an unresolved target into an incorrect bucket.

Action routing:

- `migration-artifact` stays in `migrate-research`
- `style-defect` routes to `style-evolve upgrade`
- `processor-defect` routes to engine follow-up
- `intentional divergence` routes to adjudication

Wrapper guardrails:

- `profile + config-wrapper` work must preserve the config-wrapper contract
- if a proposed migration fix would require local templates or local
  `type-variants` in a profile target, reroute or escalate instead of breaking
  the profile contract
- `journal + structural-wrapper` is a valid endpoint and must not be
  force-reduced to a thin wrapper

If a migration-side change produces no delta in the reduced cluster and no delta
in `report-core --style-file`, stop treating the cluster as migration-owned and
reroute the same bounded cluster to processor work.

## Operator Commands
Primary oracle:

```bash
node scripts/oracle.js styles-legacy/chicago-author-date.csl --json > /tmp/chicago-primary.json
```

Official style report:

```bash
node scripts/report-core.js --style chicago-author-date > /tmp/chicago-report.json
```

Temporary migrated style:

```bash
cargo run --bin citum-migrate -- styles-legacy/chicago-author-date.csl > /tmp/chicago-migrated.yaml
node scripts/report-core.js --style chicago-author-date --style-file /tmp/chicago-migrated.yaml > /tmp/chicago-report-temp.json
```

Cluster extraction:

```bash
node scripts/extract-rich-benchmark-cluster.js \
  --style chicago-author-date \
  --benchmark chicago-zotero-bibliography \
  --type entry-dictionary,entry-encyclopedia \
  --only-mismatches \
  --out-dir /tmp/chicago-entry-cluster
```

Reduced cluster rerun:

```bash
node scripts/oracle.js styles-legacy/chicago-author-date.csl \
  --json \
  --scope bibliography \
  --refs-fixture /tmp/chicago-entry-cluster/cluster-fixture.json \
  > /tmp/chicago-entry-cluster/cluster-oracle.json
```

Reduced cluster warning:

- Do not use the legacy-style `oracle.js styles-legacy/...` rerun as the
  temporary migrated-style "after" measurement when the style is already known
  to the repo and may resolve to the checked-in YAML.
- Use the reduced cluster render against `/tmp/chicago-migrated.yaml` as the
  trustworthy post-fix surface unless a fully migrated-style-aware comparator is
  available for that fixture shape.

## Output Artifacts
The extractor writes:

- `cluster-fixture.json`
- `cluster-before.json`
- `cluster-summary.json`

These are operator artifacts only. They do not redefine official reporting or hard gates.

If `--only-mismatches` cannot safely map every failing row back to a source item
id, the extractor must preserve the selected cluster and report the unresolved
row count instead of over-reducing to a non-reproducible subset.

## V1 Limits
- bibliography-only supplemental extraction
- explicit selectors only: `--type` or `--ids`
- no changes to `report-core` published JSON shape
- no heuristic cluster inference in the extractor
- `report-core --style-file` is style-file-aware for loading and reporting, but
  its citeproc benchmark step is not yet universally migrated-style-aware for
  every wrapped supplemental fixture shape

## Promoted Field Mapping

Audit of fields the note parser (`parse_note_field_hacks`) currently extracts
from CSL-JSON `note`/Extra and how they map to `InputReference`.

Extracted by csl26-bn0r / csl26-2pey as of 2026-04-10.

### Direct fields (canonical `Reference` struct)

| Note key   | `Reference` field | Notes                              |
|------------|-------------------|------------------------------------|
| `genre`    | `genre`           | Stored directly                    |
| `type`     | `ref_type`        | Overrides the top-level CSL type   |

### Extra map (stored in `Reference.extra`, consumed via `InputReference`)

| Note key            | Extra key           | Gap?                                                  |
|---------------------|---------------------|-------------------------------------------------------|
| `status`            | `status`            | Canonical — `InputReference::status()` accessor covers Monograph, CollectionComponent, SerialComponent, Standard |
| `event-place`       | `event-place`       | Promoted to `Collection.event` (WorkRelation) for paper-conference |
| `event-location`    | `event-place`       | Promoted to `Collection.event` (WorkRelation) for paper-conference |
| `event-title`       | `event-title`       | Promoted to `Collection.event` (WorkRelation) for paper-conference |
| `archive-collection`| `archive-collection`| Promoted to `ArchiveInfo.collection` in conversion    |
| `dimensions`        | `dimensions`        | No canonical field                                    |
| `original-date`     | handled by date parser | Extraction path confirmed                          |

### Classified gaps (not yet style-addressable)

- **`event-title`** — Promoted — `Collection.event` field, wired in `from_collection_component_ref` (2026-04-10).
- **`status`** — Canonical accessor available; legal metadata (e.g. `enacted`) now
  accessible via `InputReference::status()` for Monograph, CollectionComponent, SerialComponent, Standard.
- **`event-place`** / **`event-location`** — Promoted — `Collection.event` field, wired in `from_collection_component_ref` (2026-04-10).

### Change log

| Date       | Bean        | Change                                      |
|------------|-------------|---------------------------------------------|
| 2026-04-10 | csl26-2pey  | `break → continue`: recognized pairs after free-text lines now extracted |
| 2026-04-10 | csl26-bn0r  | Added field mapping audit; documented extra-stored gaps |
| 2026-04-10 | csl26-bn0r  | Promote event-title/event-place to Collection.event for paper-conference |
