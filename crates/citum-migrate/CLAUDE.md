# citum-migrate

CSL 1.0 → Citum YAML converter. Hand-authoring with this converter is the canonical path for top parent styles; pure-automatic conversion has plateaued (remaining fidelity gaps are engine-level, not converter-level).

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
- **Convergence plateau:** the converter is at its current best for automatic mode. Don't add ad-hoc fixups without confirming the gap isn't in `citum-engine` (`render/`) instead. Bias toward engine fixes over converter fixups when the failure is downstream-visible.
- **Style source block (CRITICAL):** `info.source` is only valid for CSL-derived styles (requires `csl-id`). Output must not include it for biblatex-derived or native styles.

## Workflows

```bash
./scripts/prep-migration.sh <csl-file>          # stage a parent style for hand-authoring
/style-evolve migrate                            # routed migration skill
node scripts/oracle.js styles-legacy/apa.csl    # CSL oracle (component diff)
```

## Symbol queries

`main.rs` is large — never `cat` it. Use **jcodemunch**: `get_file_outline` to map symbols within the file, `get_symbol` to read one symbol's body, `get_repo_outline` for the crate's module API across files.
