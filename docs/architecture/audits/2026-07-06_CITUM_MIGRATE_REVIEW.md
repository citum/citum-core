# citum-migrate Crate Review

Date: 2026-07-06
Branch: `audit/citum-migrate-review`
Bean: `csl26-al39`

## Scope and Method

Comprehensive code review of `crates/citum-migrate` (~25.3k lines across 47
files; ~19k lines of production code excluding test modules). Review criteria:
[DESIGN_PRINCIPLES.md](../DESIGN_PRINCIPLES.md), the crate's own `CLAUDE.md`
contract (template-authority split, locus-before-fixup), and the
2026-06 migrate audit series (fidelity locus classification, order-aware
fitness negative result).

Coverage:

- **Fully read:** `measured_citation.rs`, `lineage.rs`,
  `passes/sqi_refinement.rs` (production portion), `synthesis/citation.rs`,
  `options_extractor/processing.rs`; module outlines for `assembly.rs`,
  `template_resolver.rs`, `synthesis/core.rs`, `synthesis/operators.rs`,
  `template_compiler/mod.rs`, `evidence.rs`, `js_runtime.rs`, `lib.rs`,
  `runtime.rs`, `ir.rs`.
- **Targeted scans crate-wide:** panic-path scan (`unwrap`/`expect`/`panic!`/
  `unreachable!` outside `#[cfg(test)]`), `unsafe` scan, TODO/FIXME scan,
  `process::exit`/env-var scan, `Result<_, String>` census, threading scan.
- **Not read line-by-line:** `template_diff.rs`, `upsampler/`, `fixups/`
  internals, `base_detector.rs`, `bib_postprocess.rs`, `template_compiler/`
  submodules, remaining `options_extractor/` modules, `analysis/`,
  `output_plan.rs`, `main.rs`. These were sampled via outlines and scans only.

## Overall Assessment

The crate is in notably good hygiene for a transitional converter: **zero
`unsafe`, zero TODO/FIXME debt, and only two production panic sites** (both
invariant-carrying `expect`s in `assembly.rs`). Module documentation is
strong — `measured_citation.rs` and `lineage.rs` open with genuinely useful
contracts, and the synthesis loop is bounded by explicit budgets
(`CandidateBudget`) with an evidence sidecar for every selection. Test style
follows the BDD naming convention. The architecture matches the documented
template-authority split (synthesis authoritative, XML compilation a seed,
options extraction permanent).

The findings below are therefore mostly structural: semantic tables duplicated
against the engine, a dead output-plan surface, an inconsistent error-handling
strategy, and process-global state in candidate scoring.

## Findings

### F1 (High) — Duplicated ref-type → title-category tables vs the engine

`passes/sqi_refinement.rs::effective_title_rendering` hardcodes the
ref-type → title-category fallback mapping (periodical/serial/monograph/
component, including literal type lists like `"article-journal" |
"article-magazine" | …`) to decide which explicit rendering fields are
redundant and can be pruned. The engine owns the authoritative version of this
mapping in `citum-engine/src/values/type_class.rs` (`title_category`,
`container_title_category`, `parent_serial_title_category`), but those are
`pub(crate)`, so migrate re-implemented them.

Pruning is only sound when it is the **exact inverse** of engine defaulting:
if the two tables ever disagree for a type, SQI refinement deletes an explicit
rendering that the engine then defaults differently — a silent fidelity
regression attributable to neither crate. The engine table already encodes
subtleties migrate's copy may not track (e.g. broadcast is serial-not-
periodical for parent-serial in the engine's tests, while migrate's
container-title arm lists broadcast among periodicals).

**Recommendation:** hoist the type-classification table into
`citum-schema-style` (next to `TitleConfig`, which both crates already
consume) and have both the engine and SQI refinement call the shared
functions. Follow-up bean: `csl26-a0em`.

### F2 (Medium) — Dead `MigrationOutputPlan` variants

`lineage.rs` defines `MigrationOutputPlan::CreateEmbeddedRootAndWrapper` and
`UpgradeEmbeddedRootAndWrapper` plus `requires_multi_artifact_write()`, but no
code path in the repository constructs either variant — `output_plan()` can
only return `Standalone` or `ExistingWrapper`, and a repo-wide search finds no
other constructor. This is speculative API: match arms and tests carry the
variants without behavior behind them. Either wire them to the embedded-root
workflow they anticipate or remove them until that workflow exists.
Follow-up bean: `csl26-xshm`.

### F3 (Medium) — Process-global panic-hook swap during candidate scoring

`measured_citation.rs::catch_candidate_unwind` calls
`std::panic::take_hook`/`set_hook` around every bibliography-candidate render
to silence panic output. The hook is process-global: any future parallelism
(or a panic on another thread during the window) races the swap and can
permanently install the silent hook or misroute another thread's panic
report. It also discards the panic payload, so an engine bug that panics on a
candidate is indistinguishable from a legitimately bad candidate (scored 0).
The crate is single-threaded today, so this is latent, not active.

**Recommendation:** install a silencing hook once (`std::sync::Once`) or keep
`catch_unwind` but log the captured payload via `tracing::debug!` instead of
suppressing it. Follow-up bean: `csl26-awef`.

### F4 (Medium) — Two error-handling regimes: typed vs `String`

`lineage.rs` defines a proper `LineageError` enum, while the synthesis /
measured-selection / js-runtime half of the crate threads `Result<_, String>`
through 23 signatures. String errors preclude programmatic handling, make
"treat as keep-inferred" decisions stringly-typed, and leak formatting
concerns into logic. A small crate-level `MigrateError` (wrapping
`LineageError`, runtime, fixture, and render failure cases) would unify the
two regimes. The two `assembly.rs` `expect`s on the XML-fallback invariant
should become error returns at the same time. Follow-up bean: `csl26-oxai`.

### F5 (Low) — Scoring bookkeeping asymmetries and a vestigial tokenizer split

- `score_bibliography_entries` counts unmatched references **and** unmatched
  rendered entries in `items`, while `invalid_candidate_score` counts only
  references — candidates that fail to render are scored against a smaller
  denominator than candidates that render extra entries. Pass counts are
  compared across candidates, so denominators should be identical or the
  discrepancy documented.
- `tokenize` is a pure alias of `tokenize_normalized`, yet the doc comments
  describe citation selection keeping a "historical raw scorer" distinct from
  the normalized one. The distinction is vestigial; `token_jaccard` and
  `token_jaccard_normalized` are the same function modulo the `normalize_text`
  the caller applies. Collapse them and fix the comments.

Follow-up bean: `csl26-8hxp`.

### F6 (Low) — Registry scans and debug plumbing

- `discover_reverse_template_parent` deserializes **every embedded style** on
  each migration that lacks a parent link; `StyleRegistry::load_default()` is
  also loaded twice per resolve/promote cycle. A precomputed reverse-template
  index on the registry (or a lazy static) removes the O(corpus) YAML parse.
- Debug output is split across five `CITUM_MIGRATE_DEBUG*` env vars plus
  `eprintln!` in the synthesis modules, while `lineage.rs` already uses
  `tracing`. Unify on `tracing` with targets so `RUST_LOG` filtering works.

Follow-up bean: `csl26-gea2`.

### F7 (Info) — Diff wrappers cannot express deletions

`lineage.rs::diff_value` emits child-side additions and changes but has no
representation for "parent has this key, child must not". A migrated wrapper
therefore silently inherits any parent field the standalone form lacked.
This matches current YAML semantics (no null-delete in the schema), but it is
an invisible constraint on wrapper fidelity — worth a doc note in the module
header and a debug-log when the child drops a parent key. Folded into
`csl26-xshm`.

### F8 (Info) — Note-class citations ride an in-text-shaped pipeline

The citation synthesis path (seeds, mutation candidates, and the
bag-of-words pass metric) was built for short in-text citations. For
note-class styles the citation is bibliographic prose, and the order- and
punctuation-blind Jaccard metric **passes** visibly wrong output (the
`early-medieval-europe` repro renders author-date-shaped order, `2, 2,`
instead of `2.2`, and a leaked `accessed` — and still scores `match: true`,
18/20 passes). Per the 2026-06-14 order-aware-fitness negative result, the
fix belongs in the note-citation **seed/template**, not in scoring. Root
cause and fix design recorded on `csl26-ahxh`.

### Hardcoded fixture-coupled type lists (accepted)

`LOCAL_DEFAULT_TYPES` / `MEDIA_FULL_DATE_TYPES` in `measured_citation.rs`
couple candidate generation to the current fixture corpus. They are
documented and bounded; acceptable while the fixture set is stable, but any
fixture-corpus change must revisit them. No action.

## Positive Patterns Worth Keeping

- Bounded search everywhere: `CandidateBudget`, `MAX_SYNTHESIS_ROUNDS`,
  held-out rejection gate with explicit fallback ordering.
- Evidence sidecars (`MigrationEvidence`) capture selection provenance,
  minimization decisions, and discovered parents — this is the model for any
  future automated pipeline in the workspace.
- The regime guard (`apply_regime_guard`) documents its exact firing
  conditions in the doc comment, including what it deliberately does not do.
- `#[allow]` attributes carry `reason =` strings consistently.

## Related Open Work

- `csl26-hxhx` — XML layout compiler removal gate (unchanged by this review;
  the seed still wins selections).
- `csl26-vpae` — processing-fold materialization; root cause narrowed and fix
  design recorded on the bean during this review.
- `csl26-ahxh` — note-class citation divergence (F8).
- `csl26-4wec`, `csl26-h0rt`, `csl26-na19` — pre-existing extractor/synthesis
  beans; none are duplicated by the findings above.
