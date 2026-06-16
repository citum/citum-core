# citum-migrate

CSL 1.0 → Citum YAML converter. Hand-authoring with this converter is the canonical path for top parent styles. The sub-90 fidelity tail is **converter-dominated**, not engine-bound: see [2026-06-14 locus classification](../../docs/architecture/audits/2026-06-14_MIGRATE_FIDELITY_LOCUS_CLASSIFICATION.md). The pass-count headline is sticky because sub-90 styles carry *compounding* converter defects under a binary threshold — fixing one is correct even when the headline does not move.

## Layout

| Path | Purpose |
|---|---|
| `src/main.rs` | CLI orchestration: parse input, resolve lineage, run assembly, emit output |
| `src/assembly.rs` | Final standalone style assembly; template source selection; full-first fixup/compression boundary |
| `src/runtime.rs` | Workspace-root discovery, template resolution, source logging, stdout/debug output |
| `src/output_plan.rs` | Family-candidate routing, wrapper/evidence accounting, output-plan logging |
| `src/lib.rs` | Library entry, macro inlining, shared module exports |
| `src/passes/` | Conversion passes (run in order) |
| `src/fixups/` | Post-conversion corrections |
| `src/options_extractor/` | XML-pipeline options extraction |
| `src/synthesis/` | Template synthesis loop — **the authority and default path** (see below) |
| `src/template_compiler/` | XML layout compilation → templates (transitional synthesis **seed**, not authority) |
| `src/analysis/` | Pre/post analysis |
| `src/base_detector.rs` | Detect base/parent style for inheritance |
| `src/compilation.rs` | Final assembly — produces the XML seed candidate (`compile_from_xml`) |
| `src/js_runtime.rs` | citeproc-js bridge for oracle/fidelity checks |
| `src/template_diff.rs` | Full-template to type-variant diff encoding plus engine round-trip validation |

## Template authority

"XML pipeline" conflates three distinct things — keep them apart:

- **Synthesis loop** (`src/synthesis/`, default path via `synthesize_citation` /
  `synthesize_bibliography`) is **the template authority**: it scores candidates against
  citeproc-js output and selects the best.
- **XML layout compilation** (`compilation.rs::compile_from_xml`, `template_compiler/`) is a
  *transitional seed candidate*, not authoritative. It still wins ~20% of selections (2026-06-13:
  24/99 citation, 17/98 bibliography) and also feeds type-variant templates and note-position
  overrides merged into the synthesized result — so **fixing its output quality is valid work**
  until the removal gate in `csl26-hxhx` holds (the `xml` seed wins ≈0 selections). Removing it
  before then regresses ~1 in 5 styles. Spec:
  [OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md](../../docs/specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md).
- **XML attribute/options extraction** (`options_extractor/`) is **permanent** — the loop reads
  XML for et-al thresholds, `initialize-with`, sort keys, etc. It is *not* part of the layout
  compilation slated for removal.

## Gotchas

- **Commit scope is `migrate`, not `csl-legacy`.** `csl-legacy` is not in the allowed scope list.
- **Oracle routing:** check `originKey` before invoking any oracle. CSL oracle ≠ biblatex oracle. The `--force-migrate` flag re-runs the full conversion on an already-migrated style.
- **Locus before fixup:** before adding an ad-hoc fixup, classify the failure. Many tail gaps are genuine converter bugs (dropped variables, missing processing mode, wrong citation mode) — fix those at the source pass, not with a downstream patch. Still confirm the gap isn't in `citum-engine` (`render/`) when the data reaches the engine correctly but renders wrong; bias toward engine fixes there. The 2026-06-14 locus classification (above) records the current converter-vs-engine split.
- **Style source block (CRITICAL):** `info.source` is only valid for CSL-derived styles (requires `csl-id`). Output must not include it for biblatex-derived or native styles.

## Workflows

```bash
./scripts/prep-migration.sh <csl-file>          # stage a parent style for hand-authoring
/style-evolve migrate                            # routed migration skill
node scripts/oracle.js styles-legacy/apa.csl    # CSL oracle (component diff)
```

## Symbol queries

Use **jcodemunch** for Rust symbols: `get_file_outline` for a file, `get_symbol` for one body, and `get_repo_outline` for the crate/module API across files.
