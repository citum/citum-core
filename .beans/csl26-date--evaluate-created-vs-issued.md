---
# csl26-date
title: 'Evaluate created vs issued dates for unpublished documents'
status: todo
type: task
priority: medium
created_at: 2026-04-08T12:00:00Z
updated_at: 2026-04-08T12:00:00Z
---

# Evaluate `created` vs `issued` dates for unpublished documents

## Context
When describing unpublished archival materials (letters, manuscripts, oral histories), the field `issued` carries publication-centric baggage that does not align with archival standards (like DACS). In archival practice, the most important date is when the material was *created*, not "issued."

## Recommendation
A proposal has been made to introduce `created` (or `dateCreated`) to the schema to serve as the primary creation/origination date for unpublished (and potentially all) content.

### Why not `issued` in Citum?
- `issued` is tied historically to “publication/issue” events in CSL and BibTeX, which is the wrong mental model for archival and born‑digital material.
- Citum is an upstream, work‑centric representation; giving it a neutral, clear term like `created` allows adapters to flexibly map to downstream vocabularies (CSL `issued`, MARC 264, DACS dates, etc.) without contorting the core schema.

### Suggested Minimal Date Vocabulary for Citum
- `created`: EDTF, the primary creation/origination date of the *content* (unpublished or published).
- `published`: EDTF, when it first became publicly available as a publication (if applicable).
- Optional: `revised`, `recorded` (for AV/oral history), etc., for domain‑specific nuance.

Adapters could decide: “when exporting Citum → CSL, map `created` or `published` to CSL’s `issued` depending on type,” allowing Citum itself to remain free of CSL’s naming baggage.

## Next Steps
Evaluate whether `created` plus a small set of more specific date roles (`published`, `recorded`, etc.) fit cleanly into the rest of the current Citum date model, and consider proposing a schema migration to support these new date fields.