---
# csl26-y4o7
title: 'engine: once-only variable consumption semantics audit'
status: todo
type: task
created_at: 2026-06-10T17:28:39Z
updated_at: 2026-06-10T17:28:39Z
parent: csl26-vmcr
---

Follow-up from C2 (bean csl26-sfir). The engine renders each reference variable at most once per bibliography entry (first occurrence wins), but consumption semantics are inconsistent: a suppressed top-level leaf component does NOT consume its variable, while members of suppressed groups DO consume (and depth-1 vs depth-2 members behaved differently in zfa probes). Decide and document the intended semantics: should suppress: true components ever claim a variable? Candidates: crates/citum-engine/src/processor/rendering/grouped/. An engine-side fix would also repair existing checked-in YAML, not just fresh migrations. Evidence repro: /tmp/dup-min.yaml pattern in C2 pass notes (suppressed group [parent-serial,date] before live group [parent-serial,volume] starves the live group).
