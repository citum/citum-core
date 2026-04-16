---
# csl26-6tny
title: 'Fix oracle tooling: embedded-YAML resolution and citation shape mismatch'
status: in-progress
type: bug
priority: high
created_at: 2026-04-16T20:04:40Z
updated_at: 2026-04-16T20:15:33Z
---

Report-core and oracle-fast fail to score Citum citations correctly, producing `citum: null` for nearly every citation entry and artificially depressing fidelity scores for many parent styles.

## Root causes

### Bug: Embedded-YAML path resolution
`scripts/oracle.js` resolves Citum YAML only from `styles/`, not `styles/embedded/`. When report-core.js passes `styleYamlPath` pointing to `styles/embedded/<name>.yaml`, oracle-fast.js ignores it and falls back to the migrator output. The 4/18-citations scores for `elsevier-vancouver`, `springer-basic-brackets`, `american-medical-association`, `springer-basic-author-date` all reflect migrator output, not the embedded YAML.

## Plan
- [x] Teach `renderWithCitumProcessor` in oracle.js to look in `styles/embedded/<name>.yaml` before migrating
- [x] Confirm report-core.js picks up the corrected scores
- [x] Re-run `node scripts/report-core.js` and confirm embedded styles now score correctly
- [ ] Update `scripts/report-data/core-quality-baseline.json` if scores move upward
- [x] Tests / regression snapshot for oracle-fast comparison path
- [x] Pre-push gate (cargo fmt --check, clippy, nextest) — N/A for JS-only; run node-side tests
- [x] PR created (#527) — CI in progress

## Rationale
Without this fix, any "low fidelity" style upgrade targeting embedded or newly-added `styles/` YAMLs is working against a broken signal. Must be fixed before running a broader style-upgrade wave.

## Out of scope
- Any Citum engine or migrator changes
- Any `styles/*.yaml` edits (defer until scores are trustworthy)
- The style upgrade wave itself (follow-up bean after this lands)

## Results

Resolved oracle.js path lookup bug. Before/after report-core fidelity deltas:

| Style | Before | After |
|-------|-------:|------:|
| american-medical-association | 0.824 | 1.000 |
| chicago-shortened-notes-bibliography | 0.955 | 1.000 |
| elsevier-vancouver | 0.686 | 1.000 |
| modern-language-association | 0.566 | 0.970 |
| springer-basic-author-date | 0.902 | 1.000 |
| springer-basic-brackets | 0.804 | 1.000 |

Portfolio gate still passes (147 styles at fidelity=1.0, warnings=1 for a
pre-existing ieee concision regression unrelated to this fix).

Refactor: extracted `resolveAuthoredStylePath(stylesDir, styleName)` with
three-step lookup (styles/<name>.yaml → styles/embedded/<name>.yaml →
styles/<prefix-match>.yaml) and unit tests in scripts/oracle.test.js.

Baseline JSON unchanged — tracked styles' fidelity scores were unaffected.
