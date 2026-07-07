---
# csl26-oxai
title: Introduce crate-level MigrateError replacing Result<_, String>
status: completed
type: task
priority: normal
created_at: 2026-07-06T18:42:20Z
updated_at: 2026-07-07T11:19:56Z
parent: csl26-al39
---

Audit F4 (2026-07-06 migrate review): lineage.rs has typed LineageError while synthesis/measured-selection/js_runtime thread Result<_, String> through 23 signatures. Add a crate-level MigrateError enum (Lineage, Runtime, Fixture, Render, Parse variants), migrate the String signatures, and convert the two assembly.rs XML-fallback expect()s into error returns. Keep display text identical where callers print it.

## Summary of Changes

Added crate::error::MigrateError (Runtime, Fixture, Render, Parse variants; no Lineage
variant — nothing in this call graph touches LineageError, and an unconstructed variant
trips the `-D warnings` dead_code gate). Migrated Result<_, String> to Result<_, MigrateError>
across js_runtime.rs (9 signatures), measured_citation.rs (5), synthesis/core.rs (1),
synthesis/bibliography.rs (2), and synthesis/citation.rs (2) — 19 signatures total, all
production non-test call sites in the synthesis/measured-selection/js_runtime cluster.
Display text at every construction site is unchanged (verbatim format!/literal, just
re-wrapped in a MigrateError variant).

Converted assembly.rs's two XML-fallback expect() calls into MigrateError::Render error
returns, making select_and_process_bibliography_template, select_citation_template,
assemble_with_selection, apply_measured_citation_selection,
apply_measured_bibliography_selection, and main.rs's apply_measured_selection_pipeline all
Result-returning, propagating with `?` up to main()'s existing
Result<(), Box<dyn std::error::Error>> (composes via MigrateError's explicit
impl std::error::Error, mirroring LineageError's existing pattern). This was a deliberate
choice over swallowing: a "fatal bootstrap error" panic downgraded to a silently-ignored
condition would hide a real bug, so it hard-propagates instead. The separate
measured_selection_unavailable swallow-and-log path (real synthesize_citation/bibliography
failures) is untouched — only the two expect() sites hard-propagate.

measured_selection_unavailable's `err: String` param became `err: MigrateError` with zero
changes needed at its two call sites (they already forward `err` unchanged).

Discovered mid-implementation: MigrateError had to be fully `pub` (not `pub(crate)`) because
`synthesize_bibliography`/`synthesize_citation`/`select`/`select_bibliography` are called
from assembly.rs/main.rs, which live in the *binary* crate (a separate compilation unit from
the citum-migrate library) — `pub(crate)` does not cross that boundary.

Verified: `just pre-commit` (fmt, clippy -D warnings, nextest) passes clean — 1833/1833 tests.
End-to-end run against styles-legacy/apa.csl confirms no behavior change.
