---
# csl26-6ne6
title: Additional locator terms (legal/liturgical/regional)
status: todo
type: feature
priority: low
tags:
    - schema
    - locale
    - taxonomy
created_at: 2026-07-12T15:35:02Z
updated_at: 2026-07-12T16:02:05Z
parent: csl26-kcda
---

Add locator terms Citum's canonical locale currently lacks:
- `surah` (Quran divisions) — CSL schema#343
- `clause`, `division`, `schedule`, `sub-clause`, `subdivision`, `sub-paragraph`, `subsection` (AGLC legal citation) — CSL schema#346
- `recital` (EU legal texts, e.g. GDPR recitals) — CSL schema#412
- distinct `volume-book`/`volume-periodical` terms (German "Bande" vs "Jahrgang") — CSL schema#418

Citum's terms-keyed-by-type architecture (see #371 in the audit) should make
these straightforward additions to crates/citum-schema-style/embedded/locales/en-US.yaml.

- [ ] Design decision: add each term individually or as one locale-authoring pass
- [ ] Add terms to en-US.yaml
- [ ] Update docs/schemas if the LocatorType surface needs regenerating
