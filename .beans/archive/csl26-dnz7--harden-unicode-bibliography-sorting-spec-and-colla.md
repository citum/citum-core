---
# csl26-dnz7
title: Harden unicode bibliography sorting spec and collation options
status: completed
type: feature
priority: normal
created_at: 2026-05-01T11:02:18Z
updated_at: 2026-05-01T11:32:16Z
---

Revise docs/specs/UNICODE_BIBLIOGRAPHY_SORTING.md and citum-engine sort_support.rs/sorting.rs to address gaps identified via Perplexity analysis: (1) add mixed-script collation policy, (2) remove lowercasing in favour of collator case settings, (3) pin CollatorOptions (strength/case/punctuation/numeric), (4) add deterministic tiebreaker, (5) expand test fixtures beyond accented Latin. Spec must be readable by multilingual domain experts. Work on a new PR branch.

## Summary of Changes

- Spec rewritten for domain-expert review: Collation Policy (locale-tailored → subtag-stripping → en-US fallback chain, limitations noted), normative Collation Options table, Deterministic Tie-Breaking semantics
- sort_support.rs: CollatorOptions now explicitly sets CaseLevel::Off + AlternateHandling::Shifted + Numeric::Off; no preprocessing of source text; script reordering documented as ICU default
- processor/sorting.rs: entry-ID tiebreaker optimized to avoid clones; None IDs sort last
- 6 new unit tests: case-insensitivity, NFC/NFD equivalence, Hangul/Latin, Arabic/Latin, punctuation ignorability
- Branch: feat/unicode-sorting-hardening
