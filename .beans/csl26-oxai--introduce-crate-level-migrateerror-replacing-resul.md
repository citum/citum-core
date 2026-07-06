---
# csl26-oxai
title: Introduce crate-level MigrateError replacing Result<_, String>
status: todo
type: task
created_at: 2026-07-06T18:42:20Z
updated_at: 2026-07-06T18:42:20Z
parent: csl26-al39
---

Audit F4 (2026-07-06 migrate review): lineage.rs has typed LineageError while synthesis/measured-selection/js_runtime thread Result<_, String> through 23 signatures. Add a crate-level MigrateError enum (Lineage, Runtime, Fixture, Render, Parse variants), migrate the String signatures, and convert the two assembly.rs XML-fallback expect()s into error returns. Keep display text identical where callers print it.
