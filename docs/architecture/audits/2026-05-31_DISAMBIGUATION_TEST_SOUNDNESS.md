# Disambiguation Test Soundness Audit

**Date:** 2026-05-31
**Status:** Complete
**Spec:** [`docs/specs/DISAMBIGUATION.md`](../../specs/DISAMBIGUATION.md)
**Bean:** csl26-ucs3

## Overview

Full soundness review of automated disambiguation tests in `citum-engine` against
`docs/specs/DISAMBIGUATION.md`. Checked for unsound, vacuous, or overfit tests.
Resulted in: 1 broken test fixed, 5 suspicious tests renamed/realigned/strengthened,
3 new tests added (2 coverage-gap tests, 1 cascade-fallback test), and 1 spec
acceptance criterion corrected.

## Summary

| Verdict | Count |
|---------|-------|
| Good (no change) | 9 |
| Suspicious (renamed/realigned/strengthened) | 5 |
| Broken (rewritten) | 1 |
| New tests added | 3 |
| Spec corrections | 1 |

**Highest-priority fix:** `subsequent_et_al_thresholds_shorten_the_repeat_citation` — broken
substring-only assertions replaced with exact `assert_eq!`.

**Notable strengths:** `apa_reprint_year_suffix_attaches_to_issued_year_only` and the
§6 suppression pair are exact, spec-anchored, and resistant to overfitting.

## Spec correction

`docs/specs/DISAMBIGUATION.md` §4 listed `[x] Multilingual key generation respects
display mode`, but `render_name_for_disambiguation` does not exist in the codebase.
`disambiguation.rs` calls `to_names_vec()` which always reads `Contributor::Multilingual.original`,
ignoring the style's display mode. The acceptance criterion has been corrected to
`[ ]` pending a future implementation pass. A new test
(`multilingual_contributors_collide_on_original_family_name`) documents what IS
currently wired — original-name form drives the collision key — and includes a
`NOTE` comment describing the future transliteration-aware path.

## Implementation gaps found

| Gap | Status |
|-----|--------|
| §4 transliteration-mode collision key | Not implemented; `[ ]` restored in spec |
| §2 `givenname-disambiguation-rule: by-cite \| all-names` | Not in schema (`add_givenname: bool` only); no parameterized test possible |

## Full JSON analysis

```json
{
  "spec": "docs/specs/DISAMBIGUATION.md",
  "reviewed_files": [
    "crates/citum-engine/tests/citations.rs",
    "crates/citum-engine/tests/document.rs",
    "crates/citum-engine/tests/common/mod.rs"
  ],
  "tests": [
    {
      "name": "disambiguation_same_author_same_year_titles_follow_title_order",
      "location": "crates/citum-engine/tests/citations.rs:196",
      "spec_refs": ["§1 collision key", "§3 suffix ordering by title"],
      "intended_behavior": "Two works with the same author and issued year form one collision group and receive year suffixes assigned in lowercased-title order.",
      "what_it_does": "Builds two Smith/2020 books titled 'Alpha' and 'Beta', cites them in one cluster, asserts exact 'Smith, (2020a), (2020b)'.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguation_two_level_author_collisions_get_distinct_suffixes",
      "location": "crates/citum-engine/tests/citations.rs:212",
      "spec_refs": ["§2 cascade (names then year_suffix)"],
      "intended_behavior": "When et-al truncation collapses distinct author lists into colliding sub-groups, each sub-group is independently year-suffixed.",
      "what_it_does": "Four 1986 books (two 3-author, two 4-author) with names+year_suffix enabled; asserts exact full string where each truncation level gets its own (1986a)/(1986b).",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguation_same_year_articles_increment_suffixes",
      "location": "crates/citum-engine/tests/citations.rs:333",
      "spec_refs": ["§1", "§3"],
      "intended_behavior": "Three same-author/same-year works increment suffixes a, b, c in title order.",
      "what_it_does": "Three Ylinen/1995 articles (titles A/B/C), asserts exact 'Ylinen, (1995a), (1995b), (1995c)'.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguation_duplicate_family_names_expand_given_names_only_where_needed",
      "location": "crates/citum-engine/tests/citations.rs:346",
      "spec_refs": ["§2 given-name expansion"],
      "intended_behavior": "Given-name expansion is applied only to items whose family names collide across the colliding cites.",
      "what_it_does": "Three refs with differing years — no collision is ever triggered, so the test asserts the ABSENCE of given-name expansion.",
      "verdict": "suspicious",
      "improvement": "Mislabeled: fixture never triggers a collision. Rename to reflect 'no spurious expansion when years differ'; add a positive case where expansion IS triggered.",
      "action_taken": "Renamed to disambiguation_no_spurious_givenname_expansion_when_years_differ; wrapper announce_behavior updated. New positive test disambiguation_givenname_expansion_resolves_same_year_family_name_collision added."
    },
    {
      "name": "disambiguation_et_al_conflicts_expand_names_when_that_resolves_them",
      "location": "crates/citum-engine/tests/citations.rs:375",
      "spec_refs": ["§2 step 1 et-al expansion"],
      "intended_behavior": "When et-al truncation hides the name that distinguishes two cites, expansion past the et-al threshold resolves the collision before year suffixes are tried.",
      "what_it_does": "Two 1980 books differing in 2nd author (Brown vs Beefheart), names-only enabled, et-al min3/first1; asserts exact expected string.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguation_et_al_conflicts_fall_back_to_year_suffixes",
      "location": "crates/citum-engine/tests/citations.rs:411",
      "spec_refs": ["§2 cascade early-exit / fallback"],
      "intended_behavior": "When name expansion cannot split a group (identical author lists), the cascade falls through to year-suffix assignment.",
      "what_it_does": "Two identical-author 1980 books, names+year_suffix enabled; asserts exact 'Smith et al., (1980a), (1980b)'.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguation_initials_are_used_when_short_form_family_names_collide",
      "location": "crates/citum-engine/tests/citations.rs:443",
      "spec_refs": ["§2 given-name expansion to initials"],
      "intended_behavior": "Colliding short-form family names expand to initials; non-colliding names stay bare.",
      "what_it_does": "Five 2000 books; Doe/Doe (John/Aloysius) expand; Roe stays bare; Smith/Smith (Thomas/Ted) left unresolved because year_suffix is off.",
      "verdict": "suspicious",
      "improvement": "Freezes the ambiguous T Smith/T Smith output with no fallback enabled. Add sibling with year_suffix enabled to verify cascade resolves the Smith pair.",
      "action_taken": "New test disambiguation_year_suffix_fallback_when_givenname_expansion_fails added (same given name, year_suffix=true — exact fallback path verified). The engine resolves Thomas/Ted at data level (different given names) so the sibling uses identical given names to force the fallback."
    },
    {
      "name": "subsequent_et_al_thresholds_shorten_the_repeat_citation",
      "location": "crates/citum-engine/tests/citations.rs:474",
      "spec_refs": ["§2 (et-al thresholds; subsequent form)"],
      "intended_behavior": "First cite shows all authors (below min); a repeat cite applies the more aggressive subsequent_min/subsequent_use_first thresholds.",
      "what_it_does": "One 3-author 2020 book cited twice; assertions used contains()/!contains() on short substrings — never pins year, punctuation, or exact name form.",
      "verdict": "broken",
      "improvement": "Replace substring checks with exact assert_eq! on both results.",
      "action_taken": "Replaced with assert_eq!(results[0], \"Doe, Smith, Jones, (2020)\") and assert_eq!(results[1], \"Doe et al., (2020)\") after capturing actual output."
    },
    {
      "name": "subsequent_et_al_configuration_uses_the_subsequent_form_on_repeat",
      "location": "crates/citum-engine/tests/citations.rs:578",
      "spec_refs": ["§2 et-al + §3 year suffix"],
      "intended_behavior": "(per wrapper) Repeat citations honor the subsequent et al. configuration.",
      "what_it_does": "Single citation cluster of three 2000 articles with year_suffix+et-al. No Position::Subsequent cite exists — the name and wrapper claim 'repeat' behavior not tested.",
      "verdict": "suspicious",
      "improvement": "Rename function and realign announce_behavior to reflect what is actually tested: year-suffix under et-al truncation.",
      "action_taken": "Renamed to disambiguation_year_suffix_assigned_when_et_al_truncation_leaves_collision; wrapper announce_behavior updated to accurately describe the test."
    },
    {
      "name": "disambiguation_conditions_expand_only_the_marked_items",
      "location": "crates/citum-engine/tests/citations.rs:679",
      "spec_refs": ["§1", "§3"],
      "intended_behavior": "Conditional disambiguation expands only specifically marked citation items.",
      "what_it_does": "Two identical-author identical-year books; plain year-suffix case with no conditional/marked-item mechanism set up.",
      "verdict": "suspicious",
      "improvement": "Name overstates scope and partially duplicates same_author_same_year case. Rename to a plain multi-author duplicate-year description.",
      "action_taken": "Renamed to disambiguation_identical_two_author_year_pair_receives_year_suffixes; wrapper updated."
    },
    {
      "name": "disambiguation_suffixes_continue_past_z",
      "location": "crates/citum-engine/tests/citations.rs:701",
      "spec_refs": ["§3 base-26 wrapping (int_to_letter)"],
      "intended_behavior": "Suffix sequence continues past z into aa, ab, ... without resetting or truncating.",
      "what_it_does": "30 Smith/1986 books; asserts the full exact sequence a..z, aa, ab, ac, ad.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "apa_reprint_issued_year_only_suffix",
      "location": "crates/citum-engine/tests/citations.rs:876",
      "spec_refs": ["§1 issued-year-only key", "§1 div-009"],
      "intended_behavior": "Year-suffix collision keying uses only the issued year; three reprints with differing original-dates all form one group and ALL receive a suffix.",
      "what_it_does": "Three Freud reprints via CSL-JSON with differing original-date, slash-grouped template; asserts exact '(1926/1967a), (1926/1967b), (1927/1967c)'.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguate_only_title_suppressed_in_note_cross_ref_position",
      "location": "crates/citum-engine/tests/citations.rs (suppression section)",
      "spec_refs": ["§6 short-title suppression via first-reference-note-number"],
      "intended_behavior": "A disambiguate-only short title is suppressed on a subsequent cite when a first-reference-note-number is available.",
      "what_it_does": "Note style; first cites of Rome/Greece show titles; subsequent Rome cite asserts exact vec.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "disambiguate_only_title_kept_when_template_lacks_note_number",
      "location": "crates/citum-engine/tests/citations.rs (suppression section)",
      "spec_refs": ["§6 suppression gated on template_uses_first_ref_note_number"],
      "intended_behavior": "When the subsequent template does NOT render a first-reference-note-number, the disambiguating title must be retained.",
      "what_it_does": "Same fixture minus the note-number component; subsequent Rome cite asserts the title is kept.",
      "verdict": "good",
      "improvement": null,
      "action_taken": null
    },
    {
      "name": "given_group_local_disambiguation_when_rendering_multilingual_groups_then_year_suffixes_restart_within_each_group",
      "location": "crates/citum-engine/tests/document.rs:822",
      "spec_refs": ["§5 disambiguate: locally (per-group suffix restart)"],
      "intended_behavior": "With per-group local disambiguation, year-suffix sequences restart at 'a' inside each BibliographyGroup and never leak across group boundaries.",
      "what_it_does": "Count of '2020a' in bibliography == 2. Does not verify the two occurrences are in different groups, or that each group has both 2020a and 2020b.",
      "verdict": "suspicious",
      "improvement": "Split bibliography into per-group blocks; assert each block independently contains 2020a AND 2020b; assert no 2020c/2020d.",
      "action_taken": "Replaced count assertion with per-group block extraction and independent 2020a/2020b assertions plus leakage guard."
    }
  ],
  "new_tests_added": [
    {
      "name": "disambiguation_givenname_expansion_resolves_same_year_family_name_collision",
      "spec_refs": ["§2 given-name expansion positive path"],
      "rationale": "Positive complement to the renamed no-spurious-expansion guard. Verifies expansion IS applied to the ambiguous Smith/Smith pair while Jones stays bare.",
      "expected": "Jones, (2000); A Smith, (2000); B Smith, (2000)"
    },
    {
      "name": "disambiguation_year_suffix_fallback_when_givenname_expansion_fails",
      "spec_refs": ["§2 cascade fallback"],
      "rationale": "When given-name expansion cannot resolve (identical given names), cascade falls through to year-suffix.",
      "expected": "Smith, (2000a), (2000b)"
    },
    {
      "name": "disambiguation_multilingual_contributors_collide_on_original_family_name",
      "spec_refs": ["§4 (partial — original-name form only)"],
      "rationale": "Verifies Contributor::Multilingual is wired into the collision-key path via to_names_vec().original. NOTE: transliteration-display-mode path not yet implemented.",
      "expected": "김, (2020a); 김, (2020b)"
    }
  ],
  "coverage_gaps_remaining": [
    {
      "spec_ref": "§4 transliteration-mode collision key",
      "observation": "render_name_for_disambiguation does not exist. disambiguation.rs calls to_names_vec() which always reads Contributor::Multilingual.original regardless of style display mode. Two refs whose originals differ but transliterations collide will NOT be grouped.",
      "recommendation": "Implement render_name_for_disambiguation in disambiguation.rs that consults config.multilingual.name_mode and calls resolve_multilingual_name; remove the [x] from DISAMBIGUATION.md acceptance criterion until done."
    },
    {
      "spec_ref": "§2 givenname-disambiguation-rule: by-cite | all-names",
      "observation": "Schema only has add_givenname: bool. The by-cite/all-names distinction from CSL is not modeled. No parameterized test is possible.",
      "recommendation": "When the rule enum is added to citum-schema-style, parameterize a test over ByCite and AllNames variants with a three-ref fixture (two colliding, one not) and assert expansion scope differs."
    }
  ],
  "summary": {
    "good": 9,
    "suspicious_fixed": 5,
    "broken_fixed": 1,
    "new_tests": 3,
    "spec_corrections": 1
  }
}
```
