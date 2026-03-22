---
# csl26-zatg
title: |-
    Migrate fr-FR and tr-TR locales to schema v2 + MF2; replace unicode escapes u201D → , \u2018 → ', \u2019 → ', \u201E → „, \u201A → ‚, \u00A0 → nbsp literal

    Spec: locales/en-US.yaml and locales/de-DE.yaml as reference for v2 structure.
    No Rust changes needed.
status: in-progress
type: task
created_at: 2026-03-22T15:14:48Z
updated_at: 2026-03-22T15:14:48Z
---

Two related cleanup items in one PR:

1. Migrate locales/fr-FR.yaml and locales/tr-TR.yaml to schema v2 + MF2 (matching en-US/de-DE):
   - Add locale-schema-version + evaluation header
   - Add messages: section with MF2 plural syntax
   - Add date-formats:, grammar-options:, legacy-term-aliases: sections

2. Replace unicode escape sequences with literal characters in en-US.yaml and de-DE.yaml (and write fr-FR/tr-TR correctly from the start):
   - \u2013 → –, \u201C → ,
