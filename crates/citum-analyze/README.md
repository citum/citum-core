# citum-analyze

Internal corpus-analysis tooling for the `styles-legacy/` CSL 1.0 collection.
Provides static census and structural comparison modes that feed migrate
converter work, style-preset discovery, and taxonomy authoring. Not published.

## Binaries

| Binary | Purpose |
|---|---|
| `citum-analyze` | Five corpus-census modes (see below) |
| `citum-batch-test` | Per-style migrate + engine pass/fail across the full corpus |

## citum-analyze modes

All modes accept a path to the `styles-legacy/` directory as the first argument.
Append `--json` to route structured output to stdout (parseable by `jq`).

### Default — feature-frequency census

```bash
cargo run --bin citum-analyze -- styles-legacy/
cargo run --bin citum-analyze -- styles-legacy/ --json | jq '.top_features[:10]'
```

Reports raw CSL attribute and element frequencies across all independent styles.
Useful for a quick corpus overview.

### `--rank-parents` — parent/dependent ranking

```bash
cargo run --bin citum-analyze -- styles-legacy/ --rank-parents --json
cargo run --bin citum-analyze -- styles-legacy/ --rank-parents --format author-date --json
```

Ranks parent styles by how many dependent styles reference them. Feeds
[`docs/reference/STYLE_PRIORITY.md`](../../docs/reference/STYLE_PRIORITY.md) and
informs which independent styles to prioritize in the converter backlog.

Filters: `--format author-date|numeric|note|label`.

### `--quantify-savings` — preset / locale-override savings estimate

```bash
cargo run --bin citum-analyze -- styles-legacy/ --quantify-savings --json | jq '.summary'
```

Estimates how many CSL styles presets, aliases, and locale overrides can absorb.
Feeds the preset-fidelity bean and [`docs/specs/STYLE_TAXONOMY.md`](../../docs/specs/STYLE_TAXONOMY.md).

### `--identify-profiles` — journal-profile candidate audit

```bash
cargo run --bin citum-analyze -- styles-legacy/ --identify-profiles --json | jq '.summary'
```

Audits the hardcoded shortlist of journal-profile candidates (defined in
`src/profile_discovery.rs`) against the live corpus. Each candidate is run
through the migrate pipeline in-process and Jaccard-matched against
[`StyleBase::all()`](../citum-schema/). Feeds
[`docs/specs/STYLE_TAXONOMY.md`](../../docs/specs/STYLE_TAXONOMY.md) and
architecture audit records.

### `--coverage-gap` — corpus-wide converter-gap analysis

```bash
cargo run --bin citum-analyze -- styles-legacy/ --coverage-gap
cargo run --bin citum-analyze -- styles-legacy/ --coverage-gap --json | jq '.prioritized_gaps[:10]'
cargo run --bin citum-analyze -- styles-legacy/ --coverage-gap --json | jq '.preset_families[:5]'
```

Runs every independent style in `styles-legacy/` through the migrate XML
compilation pipeline in-process, then computes two reports:

**Prioritized converter gaps** — CSL features present in the legacy source that
are absent from migrate's compiled output, ranked by how many independent styles
are affected. This is the primary feed for `citum-migrate` converter work: "fix
construct X to recover N styles."

**Preset-family clusters** — independent styles whose compiled semantic set
closely matches a Citum base style (Jaccard similarity ≥ 0.65), listed per base.
Data-driven extension of `--identify-profiles` from a hardcoded shortlist to the
full corpus.

> **Note on false positives.** The gap list uses set difference as a *heuristic*
> for "dropped by migrate." Some features are intentionally normalized away by
> `fixups` (e.g. locator citation injection). The list is a triage aid, not a
> ground-truth bug list. Walking all macros (not only reachable ones) may also
> include a small number of dead-macro features.

## citum-batch-test

```bash
cargo run --bin citum-batch-test -- styles-legacy/ --json | jq '{total,migration_success,migration_failed}'
cargo run --bin citum-batch-test -- styles-legacy/ --sample 300 --verbose
cargo run --bin citum-batch-test -- styles-legacy/ --json > /tmp/batch.json
```

Runs every `.csl` file through the full in-process pipeline:

1. Parse legacy CSL XML (`csl_legacy::parser::parse_style`).
2. Compile templates (`citum_migrate::compilation::compile_from_xml`).
3. Assemble a minimal `citum_schema::Style` and validate it via `serde_yaml` roundtrip.
4. Instantiate `citum_engine::Processor::new` to verify engine acceptance.

Reports per-category counts and error-type breakdowns. All styles in the
directory are tested (including `dependent/`); dependent styles are expected to
fail at step 1 since they contain no layouts.

Options:

| Flag | Effect |
|---|---|
| `--verbose` | Print each style's result as it runs |
| `--sample N` | Test a pseudo-random sample of N styles |
| `--json` | Emit final `BatchResults` as JSON |

## How this feeds the pipeline

| Mode | Consumer |
|---|---|
| `--rank-parents` | `docs/reference/STYLE_PRIORITY.md` — which parents to support first |
| `--quantify-savings` | Preset/alias taxonomy decisions |
| `--identify-profiles` | `docs/specs/STYLE_TAXONOMY.md`, architecture audits |
| `--coverage-gap` | `citum-migrate` converter backlog (gap list); preset registry seed (family clusters) |
| `citum-batch-test` | Corpus regression check after converter changes |

For iterative converter work (fixing a specific gap and re-testing), see the
`/migrate-research` skill — that skill owns the fix-and-measure loop. The tools
here are static census only.
