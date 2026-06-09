---
# csl26-outu
title: Add CNE Chicago multilingual test data and style variant
status: completed
type: task
priority: normal
created_at: 2026-06-09T18:33:51Z
updated_at: 2026-06-09T18:40:33Z
---

Add East Asian multilingual citation fixtures (Chinese article, Korean book, Japanese book) in Citum YAML, a chicago-notes-18th-cne.yaml style variant with three-part Pattern multilingual rendering, and three engine tests.

## Summary of Changes

- : three real-world East Asian citations (Chinese journal article by Hua Linfu, Korean book by Kang U-bang, Japanese book by Abe Yoshio + Kaneko Hideo) in Citum YAML format with full multilingual structure.
- : CNE variant of chicago-notes-18th; only the `multilingual` option differs — uses `Pattern` mode for three-part title rendering (romanized + original + [translated]) plus `use-native-ordering: true` for Han/Hangul scripts.
- : three new tests verifying three-part title rendering. Author name original-script appending and native ordering are engine gaps noted in the test comments.
