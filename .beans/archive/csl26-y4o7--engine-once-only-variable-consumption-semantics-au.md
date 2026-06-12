---
# csl26-y4o7
title: 'engine/migrate: once-only variable consumption semantics audit'
status: completed
type: task
priority: normal
tags:
    - engine
    - migrate
    - fidelity
created_at: 2026-06-10T17:28:39Z
updated_at: 2026-06-12T18:01:42Z
parent: csl26-vmcr
---

Random-sample migration blocker surfaced by C2 (bean csl26-sfir). Migrated bibliography templates can contain a suppressed fallback group before the live group that should render the same variable later. The engine's once-only variable consumption then decides whether the migrated style loses fields such as volume/page data, so this is tagged as both `engine` and `migrate`.

The engine renders each reference variable at most once per bibliography entry (first occurrence wins), but consumption semantics are inconsistent: a suppressed top-level leaf component does NOT consume its variable, while members of suppressed groups DO consume it. Depth-1 vs depth-2 members also behaved differently in zfa probes.

Decide and document the intended semantics: should `suppress: true` components ever claim a variable? Candidate code area: `crates/citum-engine/src/processor/rendering/grouped/`. An engine-side fix would repair existing checked-in YAML, not just fresh migrations.

Evidence trail: archived bean csl26-sfir records the C2 fix and routes the residual engine consumption-semantics question here. Reconstruct a checked-in regression fixture from the failing pattern before implementation: a suppressed group containing `parent-serial`/date before a live group containing `parent-serial`/volume can starve the live group.


## Resolution - 2026-06-12

Defined the intended semantics in `docs/specs/TEMPLATE_RENDERING_SEMANTICS.md`: variable-once is first-visible-occurrence semantics. Suppressed components and suppressed groups do not claim variables; group child consumption is transactional and only commits when the group emits visible output.

Engine fix: `TemplateComponentTracker` is cloned for group evaluation and merged only for visible groups. Explicit `suppress: true` now short-circuits before value extraction for generic template components and before child evaluation for groups.

Regression coverage added in `crates/citum-engine/src/processor/tests.rs`:
- suppressed top-level component before live same-variable component
- suppressed fallback group before live `parent-serial`/`volume`/`page` group
- depth-1 and depth-2 suppressed group children
- visible first occurrence still suppresses later duplicates

Verification:
- `cargo nextest run -p citum-engine`: 804 passed
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run`: 1571 passed
- `node scripts/oracle.js styles-legacy/zeitschrift-fur-allgemeinmedizin.csl --json --force-migrate`: 20/20 citations, 38/38 bibliography
- `node scripts/report-migrate-sqi.js --styles zeitschrift-fur-allgemeinmedizin,brazilian-journal-of-psychiatry`: 2/2 styles >=90%, combined mean 95.7
- `node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610 --out /tmp/migrate-random-csl26-y4o7.json`: attempted, stopped after several minutes because it was no longer a bounded check for this turn
