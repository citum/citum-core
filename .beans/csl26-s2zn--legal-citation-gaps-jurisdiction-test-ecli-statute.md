---
# csl26-s2zn
title: Legal citation gaps (jurisdiction-aware rendering, ECLI, statute amendments)
status: todo
type: feature
priority: low
tags:
    - schema
    - taxonomy
created_at: 2026-07-12T15:35:28Z
updated_at: 2026-07-12T16:02:13Z
parent: csl26-kcda
---

Legal-citation-specific gaps found against Citum's existing legal-citation
data model (jurisdiction/reporter/docket-number/authority fields, legal_case/
legislation terms already present — see #353 in the audit, already addressed):

- No template-level conditional test on `jurisdiction` — TemplateConditionField
  (17 values) doesn't include it, so styles can't branch on jurisdiction the
  way they can on author/title/genre/etc. — CSL schema#320 (and #62, older
  duplicate stub)
- No dedicated ECLI (European Case Law Identifier) field — CSL schema#131
- No mechanism to cite a specific amended version of a statute (amendment
  date, gazette-of-publication citation) — CSL schema#339
- No general `identifiers` container for identifier types outside the named-
  field list (ECLI, ISMC, ISWC, etc.) — CSL schema#350, overlaps #131

- [ ] Design: jurisdiction-aware type/preset mechanism (NOT a new template
      conditional — see note above)
- [ ] Design: dedicated ECLI field vs generic identifiers container — decide
      before implementing either #131 or #350 to avoid two competing answers
- [ ] Statute-amendment versioning: scope with a legal-citation domain expert
