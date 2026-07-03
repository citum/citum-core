---
# csl26-l59t
title: Partial-locale messages/date_formats/legacy_term_aliases replace instead of merge
status: completed
type: bug
priority: normal
created_at: 2026-07-03T16:36:01Z
updated_at: 2026-07-03T17:18:39Z
parent: csl26-h7oc
---

Locale::from_raw_with_base (crates/citum-schema-style/src/locale/raw_conversion.rs) wholesale-replaces messages, date_formats, and legacy_term_aliases from the raw locale rather than merging them key-by-key into the base, unlike roles/locators/terms/vocab which do merge. Since these fields default to empty HashMaps when absent from a locale's YAML (serde default), a partial locale file that defines no messages: section (e.g. ar-AR.yaml, eu-ES.yaml) silently loses every en-US fallback MF2 message -- including patterns like pattern.originally-published-as, the exact class of bug csl26-6n17 was filed to fix.

Flagged by Copilot review on PR #998 (csl26-6n17). Confirmed pre-existing on main (unchanged by that PR's refactor) -- the PR's from_raw docstring was corrected to accurately describe this replace-vs-merge split rather than silently fixing the behavior, since a real fix requires deciding merge semantics for three different maps across every non-en-US locale and re-running a full fidelity sweep -- out of scope for that PR.

## Todo
- [x] Decide merge semantics for messages/date_formats/legacy_term_aliases (merge into base, raw entries override on key collision, matching roles/locators/terms behavior)
- [x] Implement in from_raw_with_base
- [x] Full report-core.js fidelity sweep across all locales (not just en-US), since this changes every partial locale's effective fallback
- [x] Update raw_conversion.rs docstring once behavior is fixed
