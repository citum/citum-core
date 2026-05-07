---
# csl26-ur17
title: Fix SQI duplication-penalty discount for diff variants
status: completed
type: bug
priority: high
created_at: 2026-05-07T11:21:07Z
updated_at: 2026-05-07T11:23:35Z
---

PR #632 partially discounted diff-variant components from concision penalties: componentPenalty and overridePenalty are correctly reduced but duplication penalties (crossScopeRepeats, fingerprint accumulators) still over-penalize surgical type-variants. Also strip __isDiff flag from fingerprint hashes and add regression tests for surgical-modify case and type-variants-only fallback robustness.

## Summary of Changes

- Track `isDiffForm` at scope level in `addVariantScopes` so nested children inside diff `add` ops are also discounted (the previous per-component `__isDiff` flag missed nested groups/items).
- Filter diff-form scopes from `flattened` (component-count budget and override density) and from `keyScopeCount` (cross-scope repeat detection). `withinScopeDuplicates`, `exactDuplicateScopes`, `nearDuplicateScopes`, and `repeatedPatterns` still iterate all scopes — parallel identical diff ops are real duplication and should still register.
- Strip `__isDiff` flag in fingerprint helpers (`fingerprintComponent`, `fingerprintScopeComponents`, `collectPatternFingerprints`) so the synthetic flag doesn't leak into hashes.
- Export `computeFallbackRobustness` for direct testing.
- Add 4 regression tests:
  - surgical diff variants don't register as cross-scope repeats and don't inflate component count
  - parallel identical diff variants do register as duplicates (real duplication caught)
  - type-variants alone satisfy fallback robustness coverage
  - `extends` short-circuit still scores 100
