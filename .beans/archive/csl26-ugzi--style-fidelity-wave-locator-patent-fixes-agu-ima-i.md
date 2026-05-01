---
# csl26-ugzi
title: 'Style fidelity wave: locator + patent fixes (AGU, IMA, INFORMS)'
status: completed
type: task
priority: normal
created_at: 2026-04-30T22:59:26Z
updated_at: 2026-04-30T23:20:25Z
---

Fix systematic fidelity gaps in 3 styles:
1. american-geophysical-union (0.981→1.0): patent type-variant uses 'number: number' which doesn't resolve for patent type; fix to 'number: patent-number'
2. institute-of-mathematics-and-its-applications (0.885→~1.0): IMA CSL suppresses locators in citations entirely; Citum style incorrectly includes 'variable: locator'; also fix bibliography et-al shorten inheritance
3. institute-for-operations-research-and-the-management-sciences (0.885→~0.96+): missing 'prefix: ", "' on locator variable causes '1962p. 23' instead of '1962, p. 23'; add type-variants for legal/webpage

- [x] Fix AGU patent type-variant number variable
- [x] Fix IMA citation template (remove locator) + bibliography shorten
- [x] Fix INFORMS locator prefix + add legal/webpage type-variants
- [x] Run oracle and verify improvements
- [x] Pre-commit gate pass

## Summary of Changes

- **american-geophysical-union** (0.981 → 1.0): fixed patent type-variant using `number: number` (wrong) → `number: patent-number`
- **institute-of-mathematics-and-its-applications** (0.885 → 1.0): removed `variable: locator` from citation template (IMA CSL suppresses locators); added explicit `shorten: {min: 99}` to bibliography contributors to prevent global et-al settings cascading
- **institute-for-operations-research-and-the-management-sciences** (0.885 → 1.0): added `prefix: ", "` to citation locator; added `webpage` type-variant using `term: retrieved` locale term + new `month-abbr-day-year` date form; added `legal_case` type-variant stripping reporter/volume/pages (title serves as identifier)
- **Engine co-evolution**: added `MonthAbbrDayYear` date form ("Jan 15, 2024" format) to `DateForm` enum and both rendering functions in `date.rs`
