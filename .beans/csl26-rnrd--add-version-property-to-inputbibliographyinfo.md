---
# csl26-rnrd
title: Add version property to InputBibliographyInfo
status: todo
type: task
priority: low
tags:
    - schema
created_at: 2026-07-12T18:16:18Z
updated_at: 2026-07-12T18:16:18Z
parent: csl26-kcda
---

CSL schema#319 (bucket 1, partial, in the audit — most of the ask is
already met): bib.json root is already an object with metadata + a
references array ({info: InputBibliographyInfo, references: [...], sets,
custom}), matching the issue's core proposal. What's missing is a
`version` property specifically — InputBibliographyInfo currently only has
{title, author}. Raised directly by Bruce during PR review ("do we need to
add a version property then?").

- [ ] Add a `version` field to InputBibliographyInfo (crates/citum-schema-data)
