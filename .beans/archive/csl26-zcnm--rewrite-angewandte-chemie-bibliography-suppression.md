---
# csl26-zcnm
title: Rewrite angewandte-chemie bibliography suppression and chemistry-specific spine
status: completed
type: task
priority: high
created_at: 2026-03-08T00:34:45Z
updated_at: 2026-03-08T01:19:24Z
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

## Summary of Changes

- Rewrote bibliography spine: added citation-number `[N]` prefix
- Added type-templates for title suppression:
  - `article-journal`, `article-newspaper`, `article-magazine`: suppress title, show journal + year (space-joined)
  - `chapter`, `paper-conference`: suppress title, "in" connector, editor "(Eds.: ...)" format
  - `entry-encyclopedia`: suppress title, "in" connector, "Vol." prefix
  - `thesis`: suppress title, show `genre` field (e.g. "PhD thesis")
  - `patent`: author + patent number + year
- Fixed `personal_communication`: changed from `[]` suppression to rendered template
- Removed erroneous CSL `id` and `source` block (biblatex-derived styles have neither)
- Global title options changed to `emph: false` (chem-angew renders titles plain)
- Result: bibliography 17/33 → 33/33, fidelity 0.712 → 0.983, quality 0.897 → 0.881
