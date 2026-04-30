---
# csl26-r4dm
title: Author standards-backed CSE and NLM family bases for publisher profiles
status: completed
type: task
priority: normal
tags:
    - standards
    - style
    - styles
    - taxonomy
created_at: 2026-04-21T13:31:00Z
updated_at: 2026-04-30T20:44:27Z
---

`csl26-nrkn` confirmed that the Taylor & Francis and Springer CSE/NLM variants
have real standards-backed parentage, but today they still sit on public profile
handles or on generic standards styles that are too far away for safe direct
collapse.

## Goal

Create explicit CSE/NLM family bases for publisher-backed wrappers so those
profiles stop carrying standards-root behavior implicitly.

## Tasks

- [x] Decide the required base set for `cse-name-year`,
      `nlm-citation-sequence`, and `nlm-citation-sequence-brackets` derived
      publisher profiles.
- [x] Author the dedicated family base YAML needed by:
      `taylor-and-francis-council-of-science-editors-author-date`,
      `taylor-and-francis-national-library-of-medicine`, and
      `springer-vancouver-brackets`.
- [x] Re-run shared-fixture comparisons against the authored family bases.
- [x] Convert public profiles to `extends:` wrappers only after parity is
      proven on the accepted verification surface.

## Acceptance

- standards-backed family base(s) exist where the authority chain requires them
- public publisher profiles no longer need to impersonate standards roots
- wrapper reductions are evidence-stable before landing

## Related

- csl26-nrkn
- csl26-wp6y

## Summary of Changes

Work completed in commit `037c7c78` (feat(schema): implement config-only profile overrides).
Three publisher-family cores created in `styles/embedded/`:

- `taylor-and-francis-council-of-science-editors-author-date-core.yaml` — CSE name-year root
- `taylor-and-francis-national-library-of-medicine-core.yaml` — NLM brackets root
- `springer-vancouver-brackets-core.yaml` — Springer Vancouver brackets root

Public wrappers are thin config-wrappers extending the respective cores. Fidelity verified:
- T&F CSE author-date: 18/18 citations, 33/34 bibliography (33/34 is baseline from csl26-nrkn)
- T&F NLM: 18/18 citations, 34/34 bibliography
- Springer Vancouver brackets: 18/18 citations, 34/34 bibliography
