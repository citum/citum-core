---
# csl26-5y6k
title: 'Punctuation realization: migrate GB/T to semantic marks, demote remap to shim'
status: todo
type: feature
priority: normal
tags:
    - multilingual
    - punctuation
    - style
created_at: 2026-07-19T16:30:10Z
updated_at: 2026-07-19T16:30:16Z
parent: csl26-0ugp
blocked_by:
    - csl26-w6wf
---

Increment 3 (final) of the punctuation realization layer (spec §8). Requires
the `{ mark: ... }` token form and per-script `realization` override from
increment 2 (`csl26-w6wf`) — GB/T's literal full-width punctuation can't move
to semantic marks until that schema surface exists.

- Migrate embedded bilingual styles (GB/T 7714 first) from literal full-width
  punctuation (`prefix: （`, `delimiter: ，`, etc.) to semantic marks +
  `options.multilingual.realization-default: cjk` (already available from
  increment 1, `csl26-k2kp`).
- Demote `remap_to_latin_punctuation`
  (`crates/citum-engine/src/render/component.rs`) to documented
  compatibility-shim status: kept functional for external literal-authored
  bilingual styles, no longer extended to new scripts or marks (spec §7).
- Update `MULTILINGUAL.md` §3.2a to reflect the shim's demoted status.

## Acceptance criteria (from spec)

- [ ] The GB/T embedded style migrated to semantic marks matches its
      standard-derived expectations, with citeproc-js divergences registered
      where the standard and the oracle disagree.
- [ ] `remap_to_latin_punctuation` documented as a compatibility shim in
      `MULTILINGUAL.md` §3.2a; no new scripts/marks added to it going forward.

Spec: `docs/specs/PUNCTUATION_REALIZATION.md` §8 (increment 3).
