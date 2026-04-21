---
# csl26-xt7k
title: Split Springer Basic family-root behavior from public wrappers
status: todo
type: task
priority: normal
tags:
    - styles
    - taxonomy
    - springer
created_at: 2026-04-21T13:32:00Z
updated_at: 2026-04-21T13:32:00Z
---

`csl26-nrkn` confirmed that `springer-basic-brackets` has real parentage to
`springer-basic-author-date`, but the current `extends:` delta still has to
carry almost the whole file because bibliography arrays replace wholesale.

## Goal

Separate the Springer Basic family root from the public author-date and numeric
profiles, then reduce `springer-basic-brackets` to a meaningful wrapper.

## Tasks

- [ ] Decide whether the project needs a dedicated hidden Springer Basic family
      root instead of reusing the public `springer-basic-author-date` handle.
- [ ] Define the minimum bibliography/type-variant delta required by
      `springer-basic-brackets`.
- [ ] If current merge semantics remain too coarse, scope the smallest
      infrastructure follow-up that would make the wrapper meaningfully smaller.
- [ ] Land the wrapper conversion only when the reduced YAML is materially
      smaller and preserves current accepted output.

## Acceptance

- the family-root contract for Springer Basic is explicit
- `springer-basic-brackets` is either a meaningful wrapper or has a concrete
  infrastructure blocker bean attached

## Related

- csl26-nrkn
- csl26-ocdt
- csl26-wp6y
