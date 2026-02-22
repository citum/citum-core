---
# csl26-p0dc
title: 'Phase 0: Move csl_legacy coupling from csln_core to csln_migrate'
status: todo
type: refactor
priority: normal
created_at: 2026-02-22T00:00:00Z
updated_at: 2026-02-22T00:00:00Z
blocking:
    - csl26-modz
---

`csln_core` (the intended schema source-of-truth) currently imports
`csl_legacy` and `biblatex` via `crates/csln_core/src/reference/conversion.rs`.
This is a boundary violation: the schema crate should define types; the
migration crate should define conversions from external formats.

## Coupling to remove

File: `crates/csln_core/src/reference/conversion.rs`

* `From<csl_legacy::csl_json::Reference> for InputReference`
* `From<csl_legacy::csl_json::DateVariable> for EdtfString`
* `From<Vec<csl_legacy::csl_json::Name>> for Contributor`
* `InputReference::from_biblatex()` (imports biblatex::{Chunk, Entry, Person})

## Target

Move these impls/functions to `crates/csln_migrate/` as free functions or
trait impls in a new `conversion` module. Update all call sites.

Remove from `csln_core/Cargo.toml`:
* `csl_legacy = { path = "../csl_legacy" }`
* `biblatex = "0.11"`

## Risk

Medium. Requires auditing all call sites across the workspace that use the
`From` impls or `from_biblatex`. Run full test suite after the change.

Refs: csl26-modz, docs/architecture/CITUM_MODULARIZATION.md
