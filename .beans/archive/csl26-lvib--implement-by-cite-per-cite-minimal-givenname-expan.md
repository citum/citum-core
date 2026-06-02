---
# csl26-lvib
title: Implement by-cite per-cite minimal givenname expansion
status: completed
type: feature
priority: normal
created_at: 2026-06-02T17:38:03Z
updated_at: 2026-06-02T18:21:38Z
---

CSL's by-cite (default since 1.0.1) requires per-cite minimal expansion: each cite expands only the minimum given-name subset needed to resolve its own collision. The engine currently expands given names globally for all references in a collision group (all-names behavior). Engine: apply_group_hints sets expand_given_names on every reference in a group; to implement by-cite it would need to compute per-cite expansion lazily, comparing each rendered cite against all other cites currently in scope rather than the full bibliography collision group. Spec: docs/specs/DISAMBIGUATION.md §2.1. Schema field GivennameRule::ByCite already exists (csl26-4ada); this bean covers the engine behavior only.

## Completion

Implemented in `codex/csl26-lvib-by-cite-givenname`: citation rendering now overlays current-citation name-expansion hints for `GivennameRule::ByCite`, while `AllNames` keeps global expansion. Native regressions distinguish the two rules on the same fixture, and the disambiguation spec/reference/audit plus CSL fixture intake metadata were updated.
