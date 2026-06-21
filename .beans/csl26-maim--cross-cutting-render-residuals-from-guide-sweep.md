---
# csl26-maim
title: Cross-cutting render residuals from guide sweep
status: todo
type: task
priority: normal
created_at: 2026-06-21T10:49:53Z
updated_at: 2026-06-21T10:49:53Z
---

Engine-level rendering residuals that recurred across multiple styles in the guide-conformance sweep (csl26-53zy / PR #946). These are cross-cutting (not per-style YAML) and were deferred as risky/global. Prior context in the closed beans csl26-hxqq (disambiguation + author substitution) and csl26-9a89 (delimiter/format edge cases).

- Substitute editor label renders `(eds.)`; guides want `Eds.` / `, eds.` (IEEE, AMA, MLA). The RoleLabel `text-case` option added in this PR is the building block.
- `entry-suffix` DOI/URL terminal-period policy is per-engine, not per-style: IEEE wants the period after a DOI, AMA/NLM do not, MLA wants it after a URL. Needs a per-style or per-component control.
- Disambiguation strategy: year-suffix appears where guides use initials/added names (MLA, APA same-surname); same-surname/year ordering wrong (Garcia 2019b before 2019a).
- Proper-noun preservation under sentence-case (Springer Vancouver lowercases `Cambridge`->`cambridge`).
- Page-range en-dash vs hyphen (AMA uses hyphen; Citum emits en-dash).

Detail per-style in docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md.
