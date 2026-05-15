---
# csl26-hnpf
title: 'Engine: treat suppress-author as seen for integral name memory'
status: completed
type: bug
priority: normal
created_at: 2026-05-15T11:09:23Z
updated_at: 2026-05-15T11:19:23Z
parent: csl26-ycyp
---

In annotate_integral_name_states, citations with suppress_author:true should update the seen-map (same as integral mode). Currently only CitationMode::Integral is tracked. File: crates/citum-engine/src/processor/document/integral_names.rs

## Summary of Changes\n\nIn annotate_integral_name_states, changed guard from CitationMode::Integral-only to also fire when suppress_author:true, so suppress-author citations mark the work as seen for subsequent integral name-memory lookups.
