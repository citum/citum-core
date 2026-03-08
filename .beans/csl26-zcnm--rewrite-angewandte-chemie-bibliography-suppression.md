---
# csl26-zcnm
title: Rewrite angewandte-chemie bibliography suppression and chemistry-specific spine
status: todo
type: task
priority: high
created_at: 2026-03-08T00:34:45Z
updated_at: 2026-03-08T00:34:45Z
---

`angewandte-chemie` still carries chemistry-specific bibliography behavior that
looks more like a migration artifact than a stable declarative style spine. The
recent compatibility-floor work improved fidelity, but the suppression logic and
entry structure remain hard to reason about and likely too brittle for future
chemistry-style reuse.

Rewrite the bibliography suppression behavior so it is explicit, reviewable, and
aligned with the actual chemistry authority rather than accumulated template
patches. Treat this as both a style cleanup and a pattern-extraction exercise:
if Angewandte needs a chemistry-specific spine, make that structure clear enough
to reuse in related styles instead of burying the behavior in ad hoc overrides.
