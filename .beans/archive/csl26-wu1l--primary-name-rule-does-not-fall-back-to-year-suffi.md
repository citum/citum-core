---
# csl26-wu1l
title: primary-name givenname rule does not fall back to year-suffix when primary names are identical
status: completed
type: bug
priority: normal
created_at: 2026-06-02T18:52:04Z
updated_at: 2026-06-02T19:27:10Z
---

When givenname-disambiguation-rule: primary-name is active and expanding the first author's given name does not resolve a collision (both works share an identical primary author), the engine does not fall back to year-suffix as the next cascade step. The two ambiguous citations render identically instead of receiving year suffixes.

Discovered during test soundness review of csl26-lvib (by-cite givenname expansion, PR 868). The primary_name_givenname_expansion_expands_first_author_only test pins this broken output: both ASTHMA-A and ASTHMA-B render as 'A Asthma, Bronchitis, et al., (1990)' with no suffix.

Expected: 'A Asthma, Bronchitis, et al., (1990a)' and 'A Asthma, Bronchitis, et al., (1990b)'.

Spec ref: docs/specs/DISAMBIGUATION.md section 2 -- givenname expansion is one step in the cascade; year-suffix must be applied when the expansion step does not uniquely identify each work.

## Summary of Changes

Fixed try_apply_combined_resolution (and the subgroup path in try_apply_name_partitions) in crates/citum-engine/src/processor/disambiguation.rs. Added primary_only: bool to check_givenname_resolution / append_givenname_resolution_key. When primary_givenname_only is active and full-expansion check reports resolution but primary-only check does not, the engine now calls apply_year_suffix_for_group with expand_given_names=true and min_names_to_show retained rather than skipping suffix entirely.

Updated integration test disambiguation_primary_name_givenname_expansion to assert the corrected output. Added unit test test_primary_name_identical_primary_falls_back_to_year_suffix (hints-level). Added integration test disambiguation_primary_name_givenname_expansion_resolves_distinct_primary_authors (success path, no suffix -- test-soundness coverage gap).

Updated docs/specs/DISAMBIGUATION.md acceptance criteria and changelog.
