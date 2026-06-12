---
# csl26-y4o7
title: 'engine/migrate: once-only variable consumption semantics audit'
status: todo
type: task
priority: normal
tags:
    - engine
    - migrate
    - fidelity
created_at: 2026-06-10T17:28:39Z
updated_at: 2026-06-12T17:25:53Z
parent: csl26-vmcr
---

Random-sample migration blocker surfaced by C2 (bean csl26-sfir). Migrated bibliography templates can contain a suppressed fallback group before the live group that should render the same variable later. The engine's once-only variable consumption then decides whether the migrated style loses fields such as volume/page data, so this is tagged as both `engine` and `migrate`.

The engine renders each reference variable at most once per bibliography entry (first occurrence wins), but consumption semantics are inconsistent: a suppressed top-level leaf component does NOT consume its variable, while members of suppressed groups DO consume it. Depth-1 vs depth-2 members also behaved differently in zfa probes.

Decide and document the intended semantics: should `suppress: true` components ever claim a variable? Candidate code area: `crates/citum-engine/src/processor/rendering/grouped/`. An engine-side fix would repair existing checked-in YAML, not just fresh migrations.

Evidence trail: archived bean csl26-sfir records the C2 fix and routes the residual engine consumption-semantics question here. Reconstruct a checked-in regression fixture from the failing pattern before implementation: a suppressed group containing `parent-serial`/date before a live group containing `parent-serial`/volume can starve the live group.
