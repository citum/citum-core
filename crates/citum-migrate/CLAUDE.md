# citum-migrate

CSL 1.0 → Citum YAML converter. Hand-authoring with this converter is the canonical path for top parent styles. The sub-90 fidelity tail is **converter-dominated**, not engine-bound: see [2026-06-14 locus classification](../../docs/architecture/audits/2026-06-14_MIGRATE_FIDELITY_LOCUS_CLASSIFICATION.md). The pass-count headline is sticky because sub-90 styles carry *compounding* converter defects under a binary threshold — fixing one is correct even when the headline does not move.

## Layout

| Path | Purpose |
|---|---|
| `src/main.rs` | CLI entry (~45K — large, navigate via jcodemunch) |
| `src/lib.rs` | Library entry, pipeline orchestration |
| `src/passes/` | Conversion passes (run in order) |
| `src/fixups/` | Post-conversion corrections |
| `src/options_extractor/` | XML-pipeline options extraction |
| `src/template_compiler/` | Template generation (LLM-authored for top parents) |
| `src/analysis/` | Pre/post analysis |
| `src/base_detector.rs` | Detect base/parent style for inheritance |
| `src/compilation.rs` | Final assembly |
| `src/js_runtime.rs` | citeproc-js bridge for oracle/fidelity checks |
| `src/template_diff.rs` | Reviewer-facing template diffs |

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

`main.rs` is large — never `cat` it. Use **jcodemunch**: `get_file_outline` to map symbols within the file, `get_symbol` to read one symbol's body, `get_repo_outline` for the crate's module API across files.
