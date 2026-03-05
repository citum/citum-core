---
# csl26-paok
title: Name formatting architecture redesign
status: in-progress
type: feature
priority: high
created_at: 2026-03-05T13:48:49Z
updated_at: 2026-03-05T13:48:49Z
---

Remove ModeDependent from AndOptions; add NameForm enum (Full|FamilyOnly|Initials) to ContributorConfig; migrate all affected styles; write architecture doc. Breaking change — implement on a PR branch.

## Tasks
- [ ] Add NameForm enum to citum-schema contributors.rs
- [ ] Add name_form: Option<NameForm> to ContributorConfig + update merge()
- [ ] Remove AndOptions::ModeDependent variant
- [ ] Remove ModeDependent resolver loop in contributor.rs
- [ ] Implement NameForm dispatch in given-name rendering path
- [ ] Migrate apa-7th.yaml: replace mode-dependent with integral.options.contributors.and
- [ ] Migrate chicago-author-date.yaml: add name_form per-position
- [ ] Migrate jm-turabian-multilingual.yaml: audit and add name_form
- [ ] Audit all other styles for name_form needs
- [ ] Write docs/architecture/NAME_FORMATTING.md
- [ ] Oracle + batch-aggregate verification; update baseline if scores improve
