---
# csl26-1htc
title: Add givenname-disambiguation-rule to Disambiguation schema
status: scrapped
type: feature
priority: normal
created_at: 2026-06-02T13:49:07Z
updated_at: 2026-06-02T13:49:35Z
---

The Disambiguation struct (citum-schema-style/src/options/processing.rs) has no givenname_rule field mapping to CSL's givenname-disambiguation-rule attribute. The engine therefore always applies givenname expansion to all name positions (all-names behavior) rather than respecting:
- primary-name-with-initials (APA 7): only first author, initials only
- primary-name (Chicago author-date): only first author, full given name
- all-names / all-names-with-initials / by-cite

In practice, formatting (initials vs full) is driven by the contributor config's initialize-with / name-form, masking the issue. But the scoping rule (which positions trigger expansion) is not enforced.

Spec: docs/specs/CROSS_ENTRY_FIDELITY.md § Schema Gap

## Tasks
- [ ] Add GivennameDisambiguationRule enum to citum-schema-style
- [ ] Add givenname_rule: Option<GivennameDisambiguationRule> to Disambiguation struct
- [ ] Engine: use givenname_rule to scope which author positions get expand_given_names=true
- [ ] Update apa-7th.yaml: add givenname_rule: primary-name-with-initials (if not covered by contributor config)
- [ ] Update chicago-author-date.yaml: add givenname_rule: primary-name
- [ ] Add rstest parameterised tests for each rule variant
- [ ] Regenerate JSON schema docs

Duplicate of csl26-4ada. Scrapping.
