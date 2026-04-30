---
# csl26-xt7k
title: Split Springer Basic family-root behavior from public wrappers
status: completed
type: task
priority: normal
tags:
    - styles
    - taxonomy
    - springer
    - style
created_at: 2026-04-21T13:32:00Z
updated_at: 2026-04-30T21:03:19Z
---

`csl26-nrkn` confirmed that `springer-basic-brackets` has real parentage to
`springer-basic-author-date`, but the current `extends:` delta still has to
carry almost the whole file because bibliography templates and many
`type-variants` are expressed through replace-whole array/map structures. Under
the current merge contract, objects deep-merge, but arrays and explicit `null`
replace inherited content wholesale, so a localized child change can force
nearly complete restatement of the inherited bibliography block.

Alternative design draft: `docs/specs/CONFIG_ONLY_PROFILE_OVERRIDES.md`.

## Goal

Separate the Springer Basic family root from the public author-date and numeric
profiles, then reduce `springer-basic-brackets` to a meaningful wrapper or
explicitly scope the inheritance-model follow-up needed to make that possible.

## Tasks

- [x] Decide whether the project needs a dedicated hidden Springer Basic family
      root instead of reusing the public `springer-basic-author-date` handle.
- [x] Define the minimum bibliography/type-variant delta required by
      `springer-basic-brackets`.
- [x] Decide whether the project should add finer-grained inheritance/override
      mechanics for bibliography/type-variant structures, or keep the current
      merge semantics and accept larger wrapper YAML for this family.
- [x] If the current merge model remains in place, scope the smallest
      infrastructure follow-up bean that would change the merge/override model
      enough to make the wrapper materially smaller.
- [x] Land the wrapper conversion only when the reduced YAML is materially
      smaller and preserves current accepted output.

## Acceptance

- the family-root contract for Springer Basic is explicit
- `springer-basic-brackets` is either a materially smaller wrapper under the
  current model or has a concrete infrastructure bean that changes the
  bibliography/type-variant merge/override model

## Related

- csl26-nrkn
- csl26-ocdt
- csl26-wp6y

## Summary of Changes

**Decision 1 — No hidden family root.** The two cores differ in processing
mode, citation template, and bibliography coverage. A shared root would only
save scalar options — not worth the indirection.

**Decision 2 — Minimum delta already achieved.** The embedded layer already
has the thinnest possible wrappers (2 option fields each). The no-et-al
variants cannot meaningfully thin-wrap the full cores.

**Decision 3 — Accept current merge semantics.** Arrays replace wholesale;
objects deep-merge. No change to merge mechanics in this PR.

**Decision 4 — Follow-up bean created** for finer-grained array/map override
mechanics (low priority, not blocking).

**Decision 5 — Fix landed.** Removed `extends: springer-basic-author-date`
from `styles/springer-basic-brackets-no-et-al.yaml` (wrong base: numeric
style must not extend an author-date root). Made standalone by adding
`strip-periods`, `punctuation-in-quote`, `hanging-indent`, `entry-suffix`,
plus explicit `webpage` and `legal-case` type-variants previously inherited.
Oracle: 18/18 citations, 34/34 bibliography.
