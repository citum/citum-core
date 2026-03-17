---
# csl26-zy07
title: 'StyleInfo: add short_name + edition fields, convert Turabian and APA 6th'
status: todo
type: feature
priority: normal
created_at: 2026-03-17T10:25:53Z
updated_at: 2026-03-17T10:38:32Z
blocked_by:
    - csl26-fsjy
---

Add short_name and edition to StyleInfo for wizard discoverability, backfill on all well-known styles, and convert missing styles that benefit immediately.

## Tasks

- [ ] Add `short_name: Option<String>` and `edition: Option<String>` to `StyleInfo` in `citum-schema-style`
- [ ] Backfill `short_name` (and `edition` where relevant) on all well-known styles in `styles/`
- [ ] Convert APA 6th edition (source: `styles-legacy/apa-6th-edition.csl`; differs from
  7th in et-al thresholds, date format, and DOI display)
- [ ] Hand-author Turabian Notes-Bibliography — **blocked by csl26-fsjy** (two-level
  preset design). Turabian will likely become a variant of the chicago-notes-18th
  style preset rather than a standalone YAML file. Defer until that design is
  resolved. (Base: `styles/chicago-notes.yaml`; deviations: no ibid. by default,
  student-paper title page handling, slightly different footnote punctuation per
  Kate L. Turabian 9th ed.)
- [ ] Update `citum.schema.json` (run `cargo run --bin citum -- schema > citum.schema.json`)

## Backfill targets (short_name + edition where applicable)

All styles with a widely-known short name. At minimum:

| File | short_name | edition |
|------|-----------|---------|
| apa-7th.yaml | APA | 7th |
| chicago-author-date.yaml | Chicago Author-Date | 18th |
| chicago-notes.yaml | Chicago Notes | 18th |
| chicago-notes-bibliography-17th-edition.yaml | Chicago Notes | 17th |
| chicago-shortened-notes-bibliography.yaml | Chicago Shortened Notes | 18th |
| mhra-notes.yaml | MHRA | 4th |
| modern-language-association.yaml | MLA | 9th |
| harvard-cite-them-right.yaml | Harvard | — |
| elsevier-harvard.yaml | Elsevier Harvard | — |
| ieee.yaml | IEEE | — |
| nature.yaml | Nature | — |
| elsevier-vancouver.yaml | Vancouver | — |
| american-medical-association.yaml | AMA | — |
| oscola.yaml | OSCOLA | 4th |

Journal-specific styles (Elsevier, Springer, NLM variants, etc.) intentionally
omit short_name — their title IS their identity.

## Context
Driven by the Style Wizard v2 axis navigator (citum-hub#29). The wizard's
'Closest match' banner needs concise, non-ambiguous labels. StyleInfo.title
is too verbose ('MLA Handbook 9th edition (in-text citations)') and embeds
edition information as free text. Two fields are cleaner:
- `short_name`: family label — 'APA', 'Chicago Notes', 'MLA', 'Vancouver'
- `edition`: qualifier — '7th', '18th edition', '9th' (omit for journal styles)
Wizard composes them: 'APA 7th', or just 'MLA' when no edition is present.
Search on short_name groups the family; edition disambiguates within it.
Already a live issue: chicago-notes.yaml vs chicago-notes-bibliography-17th-edition.yaml;
will be acute once apa-6th.yaml is added alongside apa-7th.yaml.

## Spec reference
citum-hub specs/STYLE_WIZARD_V2.md — Step 3 Style Navigator, 'Closest match' banner
