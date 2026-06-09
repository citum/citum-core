---
# csl26-d2af
title: name-mode pattern ignores segments for names
status: completed
type: bug
priority: normal
created_at: 2026-06-09T21:39:40Z
updated_at: 2026-06-09T22:02:24Z
---

name-mode: pattern for contributor names returns only the transliterated form (crates/citum-engine/src/values/mod.rs ~line 400 hardcodes this); pattern segments are not applied, so the original-script form is never appended after the romanized name. CNE Chicago expects e.g. 'Hua Linfu 华林甫'. Repro: render tests/fixtures/multilingual/multilingual-cne-chicago.yaml with styles/embedded/chicago-notes-18th-cne.yaml — author renders 'Linfu Hua' with no 华林甫. See TODO(bean:) in crates/citum-engine/tests/multilingual.rs.

## Summary of Changes

FlatName gained an original_script field (citum-schema-data/src/reference/contributor.rs). resolve_multilingual_name (citum-engine/src/values/mod.rs) populates it in Pattern mode when the pattern includes an original view: CJK originals render family+given with no separator (华林甫). format_single_name (values/contributor/names.rs) appends it after the assembled long-form romanized name. Verified by CNE tests in crates/citum-engine/tests/multilingual.rs asserting full Chicago 18th source examples.
