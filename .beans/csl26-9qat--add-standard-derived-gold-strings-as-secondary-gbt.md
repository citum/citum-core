---
# csl26-9qat
title: Add standard-derived gold strings as secondary GB/T authority
status: todo
type: task
priority: normal
tags:
    - fidelity
    - testing
created_at: 2026-07-17T21:06:20Z
updated_at: 2026-07-17T21:06:27Z
---

PR 1064's fidelity claim was scoped to citeproc-js byte-parity, which silently inherits source-style defects (see the Latin-punctuation bug bean). Add a second benchmark run to verification-policy for the GB/T family whose expected strings come from the GB/T 7714-2025 standard's own examples (or the upstream zotero-chinese test library's expected outputs), authority: standard, so conformance claims cannot inherit CSL-M source quirks. Generalize: any style whose verification-policy authority is citeproc-js should state that scope in fidelity claims (PR text, audit docs).
