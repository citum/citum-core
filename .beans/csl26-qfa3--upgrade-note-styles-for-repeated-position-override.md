---
# csl26-qfa3
title: Upgrade note styles for repeated-position overrides and refresh compat snapshot
status: completed
type: task
priority: normal
tags:
    - styles
    - compatibility
created_at: 2026-03-10T18:31:26Z
updated_at: 2026-03-10T22:20:52Z
---

Follow-up after repeated-note semantics engine/migration work:

- re-migrate target note styles in `--template-source xml` mode first, and record whether `citation.subsequent` / `citation.ibid` now surface directly from migrate output
- classify each target as `migration now sufficient`, `needs YAML cleanup only`, or `still blocked by mixed-condition position logic`
- audit migrated note styles for use of `citation.subsequent` / `citation.ibid` (Chicago notes, OSCOLA, MHRA, Bluebook-like styles)
- run style upgrades where needed to express intended immediate-repeat behavior without flattening intentional style-authored distinctions
- prioritize OSCOLA, MHRA, Chicago notes, and Bluebook-like styles separately; legal-note trees may still diverge in what migrate can preserve
- run oracle batch impact and core report checks
- refresh docs/compat.html if portfolio metrics or examples change
- decide whether baseline updates are needed in dedicated baseline PR



2026-03-10 implementation notes:
- XML-mode re-migration rechecked for shipped note/legal cluster.
- Migration sufficient for repeated-position sections only: oscola, oscola-no-ibid, thomson-reuters-legal-tax-and-accounting-australia.
- Still blocked by mixed-condition position trees: chicago-notes, chicago-notes-bibliography-17th-edition, mhra-notes, mhra-notes-publisher-place, mhra-notes-publisher-place-no-url, new-harts-rules-notes, new-harts-rules-notes-label-page, new-harts-rules-notes-label-page-no-url.
- Applied safe YAML-only repeated-position overrides to oscola, oscola-no-ibid, and thomson-reuters-legal-tax-and-accounting-australia without replacing base citation templates.
- Added engine tests covering OSCOLA ibid/subsequent, OSCOLA no-ibid fallback, and Thomson Reuters subsequent locator rendering.
- Verification: cargo clippy --all-targets --all-features -- -D warnings; cargo nextest run; node scripts/oracle-batch-aggregate.js note/legal cluster; node scripts/report-core.js > /tmp/core-report.json && node scripts/check-core-quality.js --report /tmp/core-report.json --baseline scripts/report-data/core-quality-baseline.json.
- compat.html not refreshed because core report gate stayed at 146 styles with fidelity 1.0 and no baseline-facing portfolio change was required.


- Follow-on migrate work split into csl26-3go0 for the mixed-condition note position trees that still fall back to base citation templates.
- qfa3 is complete with classification, safe YAML-only overrides for migration-sufficient styles, and verification.
