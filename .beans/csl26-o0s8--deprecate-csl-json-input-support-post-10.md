---
# csl26-o0s8
title: Deprecate CSL-JSON input support (post-1.0)
status: todo
type: task
priority: deferred
created_at: 2026-03-28T14:40:08Z
updated_at: 2026-03-28T14:40:08Z
---

Post-1.0 cleanup: remove CSL-JSON as a first-class input format.

## Scope

- Remove `CslJson` from `RefsFormat` enum in `citum-cli`
- Remove the CSL-JSON content-sniffing fallback in `citum-engine/src/io.rs` (`load_bibliography_with_sets`)
- Update `RENDERING_WORKFLOW.md` deprecation note
- Add deprecation notice in 1.0 release docs

## Why defer

CSL-JSON is still the output format for oracle tooling and may be useful for users with existing CSL exports. The render-path sniffer is a low-friction shim (~20 lines). Removing pre-1.0 would break existing workflows with no migration path.

## Not in scope

- `csl-legacy` crate (CSL XML parser) — unrelated; stays for migration tooling
- `input_reference_from_biblatex` — pivot already removed (csl26-ox73)
