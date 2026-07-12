---
# csl26-d5vu
title: Small term additions (idem, part/supplement labels)
status: todo
type: task
priority: low
tags:
    - schema
    - locale
created_at: 2026-07-12T15:35:57Z
updated_at: 2026-07-12T18:14:43Z
parent: csl26-kcda
---

Genuine schema/type-level term gaps only. Scope narrowed on PR review:
#410 (`of`), #414 (`to`), and #438's `here`/`deposited` items were
REMOVED from this bean — verified via crates/citum-schema-style/src/locale/types.rs
that `GeneralTerm::Of`/`::To`/`::Here`/`::Deposited` already exist as
wired enum variants with message IDs. Those are locale-content-authoring
tasks now tracked as bucket-1 (partial) in the audit report, not schema
gaps — a maintainer or contributor can just add the term text to
en-US.yaml directly without a design decision. See docs/architecture/audits/2026-07-12_CSL_SCHEMA_ISSUE_TRIAGE.md.

Remaining genuine gaps (checked, no matching type/enum exists either):

- `idem`/"id." — for identical author-and-editor citations — CSL schema#443
- `part_number`/`supplement_number` number-variable-label term entries —
  the locator terms `part`/`supplement` exist, but unlike chapter_number/
  collection_number/number_of_pages/etc. there's no label variant for these
  two — CSL schema#445 (Citum shares this exact gap with upstream)

- [ ] Add `idem`/id. term (new GeneralTerm variant + en-US.yaml content)
- [ ] Add part_number/supplement_number label term entries (new GeneralTerm
      variants + en-US.yaml content)
- [ ] Separately (locale-content only, no design needed): author `of`,
      `to`, `here`, `deposited` term text in en-US.yaml — the type-level
      support already exists. `deposited` should also get a
      `pattern.deposited-date` MF2 composition alongside the flat term,
      matching the existing `accessed`/`retrieved` precedent
      (`pattern.accessed-date`, `pattern.retrieved-date` in en-US.yaml)
