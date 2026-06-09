---
# csl26-d2af
title: name-mode pattern ignores segments for names
status: todo
type: bug
created_at: 2026-06-09T21:39:40Z
updated_at: 2026-06-09T21:39:40Z
---

name-mode: pattern for contributor names returns only the transliterated form (crates/citum-engine/src/values/mod.rs ~line 400 hardcodes this); pattern segments are not applied, so the original-script form is never appended after the romanized name. CNE Chicago expects e.g. 'Hua Linfu 华林甫'. Repro: render tests/fixtures/multilingual/multilingual-cne-chicago.yaml with styles/embedded/chicago-notes-18th-cne.yaml — author renders 'Linfu Hua' with no 华林甫. See TODO(bean:) in crates/citum-engine/tests/multilingual.rs.
