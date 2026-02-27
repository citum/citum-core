---
# csl26-4rg8
title: 'Editor label capitalization: (Eds) vs (eds)'
status: completed
type: bug
priority: normal
created_at: 2026-02-14T15:05:24Z
updated_at: 2026-02-27T14:36:50Z
---

The processor capitalizes editor role labels when rendering contributor suffix. E.g. form=long for editor produces (Eds) instead of (eds) as expected by CSL oracle. The locale defines 'eds.' lowercase, and strip-periods correctly removes the period, but the first letter gets capitalized somewhere in the rendering pipeline. Affects springer-basic-brackets ITEM-4 and potentially other styles with editor labels.

## Summary of Changes\n\nRoot cause: hardcoded en_us() locale had 'Ed.'/'Eds.' (capital E) instead of CSL-standard 'ed.'/'eds.' (lowercase). Fixed all three forms (singular short, plural short, verb short) in citum-schema/src/locale/mod.rs. Updated 4 test assertions in citum-engine that reflected the old incorrect casing. All 314 tests pass.
