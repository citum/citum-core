---
# csl26-u1tq
title: Author dedicated Elsevier family bases for Harvard and Vancouver profiles
status: completed
type: task
priority: normal
tags:
    - bases
    - style
    - styles
    - taxonomy
created_at: 2026-04-21T13:30:00Z
updated_at: 2026-04-30T20:44:09Z
---

`csl26-nrkn` confirmed that `elsevier-harvard` and `elsevier-vancouver` are
real publisher-family profiles, but not safely reducible to today's existing
general bases.

## Goal

Create explicit Elsevier family inheritance roots so the public style handles can
become true parent-plus-deltas wrappers instead of carrying family-root behavior
themselves.

## Tasks

- [x] Decide whether Elsevier needs one author-date family base and one numeric
      family base, or a more granular split.
- [x] Author the dedicated family base YAML files from publisher-guide evidence.
- [x] Re-check `elsevier-harvard` and `elsevier-vancouver` against the new bases
      on the shared fixture surface.
- [x] Convert the public profiles to `extends:` wrappers only if the reduced
      form preserves current accepted behavior.

## Acceptance

- explicit Elsevier family base(s) exist in the repo
- public Elsevier profile handles no longer need to act as family roots
- wrapper reductions are landed only when parity is proven

## Related

- csl26-nrkn
- csl26-ocdt
- csl26-wp6y

## Summary of Changes

Work completed in commit `037c7c78` (feat(schema): implement config-only profile overrides).
Three Elsevier family cores created in `styles/embedded/`:

- `elsevier-harvard-core.yaml` — author-date family root
- `elsevier-vancouver-core.yaml` — numeric Vancouver family root
- `elsevier-with-titles-core.yaml` — numeric with-titles family root

Public wrappers (`elsevier-harvard.yaml`, `elsevier-vancouver.yaml`, `elsevier-with-titles.yaml`)
are thin config-wrappers extending the respective cores. Fidelity verified:
- elsevier-harvard: 18/18 citations, 34/34 bibliography
- elsevier-vancouver: 18/18 citations, 34/34 bibliography
- elsevier-with-titles: 18/18 citations, 34/34 bibliography
