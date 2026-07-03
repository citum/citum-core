---
# csl26-6n17
title: Single-source en-US fallback messages from embedded YAML
status: completed
type: bug
priority: normal
created_at: 2026-07-03T12:17:42Z
updated_at: 2026-07-03T16:14:01Z
parent: csl26-h7oc
---

crates/citum-schema-style/src/locale/embedded/en_us.rs::en_us_archive_messages() hardcodes ~20 MF2 message literals in Rust that Locale::en_us() uses as the fallback for the ~26/28 embedded styles with no info.default-locale. The canonical source, crates/citum-schema-style/embedded/locales/en-US.yaml's messages: section, has 70 entries and drifted out of sync — a real bug (surfaced by PR #997/csl26-q05f: pattern.originally-published-as existed in YAML but silently did nothing for no-default-locale styles until added to the Rust literal directly as a workaround).

## Why this is not a simple single-sourcing patch

An attempt to fix it by extracting the full YAML messages: section at runtime (mirroring embedded_en_us_vocab()'s existing extract_top_level_yaml_section pattern) was tried and reverted after discovering the two sources have PRE-EXISTING VALUE DRIFT, not just coverage gaps:

- role.translator.label: YAML says trans. (lowercase); the legacy hardcoded ContributorTerm says Trans. -- multiple APA bibliography test fixtures pin the old (arguably inconsistent -- role.editor.label is lowercase ed.) capitalized form.
- Chapter locator abbreviation: YAML's term.chapter-label says chap.; the legacy hardcoded en_us_locator_terms() says ch. -- MLA-style locator tests pin ch..
- test_no_date_term_resolves_long_and_short_forms broke outright: expected n.d., got no date -- a real regression, not just a stylistic difference.

MF2 messages are resolved before the legacy structured Terms/LocatorTerm fields (see Locale::resolved_role_term / resolved_general_term), so fully activating the YAML messages: section doesn't just add missing keys -- it flips precedence on every key where the two sources disagree, for every style relying on the Locale::en_us() fallback (26/28 embedded styles have no default-locale).

## Scope for this bean

- Audit every key present in both the Rust hardcoded literal's implied vocabulary (roles, locators, terms) AND the YAML messages: section for value drift, not just coverage gaps.
- Decide per-conflict which source is correct (YAML is presumably intended as canonical, but chap. vs ch. and n.d. vs no date need a real CMOS/style-guide check, not an assumption).
- Fix the outright regression (no-date term) as a prerequisite.
- Only then single-source via the extract_top_level_yaml_section pattern already used for vocab.
- Verify with a full node scripts/report-core.js sweep across all 154 embedded styles (not just Chicago) before merging, since this touches the fallback locale for the large majority of them.

## Todo
- [x] Enumerate every legacy-hardcoded term/message key with a YAML messages: counterpart; diff values
- [x] Classify each conflict: YAML-is-correct-fix-legacy vs legacy-is-correct-fix-YAML vs genuinely ambiguous (needs style-guide research)
- [x] Root-cause and fix the no-date term regression specifically
- [x] Implement single-sourcing (from_raw_with_base pattern) once conflicts are resolved
- [x] Full report-core.js sweep (154 styles) confirming no unintended fidelity regressions
- [x] Update/remove now-stale pinned test expectations (Trans./ch. etc.) with justification per change, not a blanket accept


## Summary of Changes

Landed on branch `fix/csl26-6n17-single-source-en-us` in 3 commits:

1. `92f6aef7` fix(locale): form-aware general_term for no-date — general_term
   built a form-blind message candidate id, collapsing "no date"/"n.d." to
   the same value; switched to the existing form-aware general_message_id.
2. `a35443c8` refactor(locale): derive en-US from embedded YAML — split
   from_raw into from_raw_with_base(raw, base) to break the circular
   dependency, made Locale::en_us() parse the embedded en-US.yaml
   (memoized via OnceLock) seeded from Locale::default(), and deleted the
   now-dead hardcoded constructors (en_us_archive_messages,
   en_us_role_terms, en_us_locator_terms, embedded_en_us_vocab,
   Terms::en_us, DateTerms::en_us, MonthNames::en_us). Completed the YAML
   so the round-trip is faithful: added number-formats, the date
   era/period terms (am/pm/timezone/BC/AD/BCE/CE) that had no YAML
   equivalent, fixed a real term.circa drift ("ca." -> "c." per CSL
   reference), and fixed role_message_id conflating Verb/VerbShort forms
   (added role.editor.verb-short / role.translator.verb-short). Added a
   round-trip regression test pinning the critical fallback values.
3. `ba2265b0` fix(styles): restore fidelity after locale switch — updated
   pinned tests that assumed the old drifted values (Trans./ch./
   Interviewer) now that the base locale resolves to the
   CSL-reference-correct values (trans./chap./interviewer); added
   text-case: capitalize-first to APA's translator and interviewer role
   labels (APA needs the capitalized form, engine support already existed
   via apply_label_case); removed an unconditional `message: term.edition`
   component from 3 legal-case templates (american-fisheries-society,
   sage-harvard, springer-physics-author-date) that only rendered nothing
   before by accident (the message didn't exist in the old incomplete
   locale) and is not exercised by any fixture with real edition data.

Verification: full node scripts/report-core.js sweep (154 styles, 308
entries counting dual-authority styles) shows zero regressions and a
net improvement (citations passed 3548->3552, case mismatches
14->10, chicago-notes-18th citations 62->66). `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo nextest run` (1712 tests) all pass. No citum-schema*/citum-cli
public API surface changed, so schema-gen was not required.
