---
# csl26-sdt0
title: 'Djot citations: no inline nocite modifier'
status: todo
type: feature
priority: normal
tags:
    - citation
    - parser
    - forward-compat
created_at: 2026-09-12T13:42:45Z
updated_at: 2026-09-12T13:42:45Z
parent: csl26-s2zo
---

There is no inline syntax for nocite in the djot parser. `parse_citation_content` recognises only
`+` (integral) and `-` (suppress author).

Nocite exists today only as a document-level list: `nocite: Vec<String>` on the document API
(`crates/citum-engine/src/api/document.rs`), delivered by csl26-cpb7. That covers a host passing a
list programmatically; it does not let an author mark a work inline while writing.

## Scope

- Add a modifier that marks a reference for the bibliography with no inline output.
- Glyph is not settled upstream: `!@key` was proposed in our modifier table, `~@key` by Omikhleia on
  djot#32, and jgm has not committed either way ("not sure about that"). Do not implement a glyph
  before djot#32 settles it — the risk is shipping syntax the spec then contradicts.
- Route the parsed items into the same path as the existing document-level `nocite` rather than a
  second mechanism.
- Tests: a nocite-marked key appears in `bibliography.entries` and produces no citation output.

## Origin

https://github.com/jgm/djot/issues/32 — nocite is raised repeatedly in the thread and remains open.
