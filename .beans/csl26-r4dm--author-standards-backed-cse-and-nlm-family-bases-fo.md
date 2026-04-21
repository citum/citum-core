---
# csl26-r4dm
title: Author standards-backed CSE and NLM family bases for publisher profiles
status: todo
type: task
priority: normal
tags:
    - styles
    - taxonomy
    - standards
created_at: 2026-04-21T13:31:00Z
updated_at: 2026-04-21T13:31:00Z
---

`csl26-nrkn` confirmed that the Taylor & Francis and Springer CSE/NLM variants
have real standards-backed parentage, but today they still sit on public profile
handles or on generic standards styles that are too far away for safe direct
collapse.

## Goal

Create explicit CSE/NLM family bases for publisher-backed wrappers so those
profiles stop carrying standards-root behavior implicitly.

## Tasks

- [ ] Decide the required base set for `cse-name-year`,
      `nlm-citation-sequence`, and `nlm-citation-sequence-brackets` derived
      publisher profiles.
- [ ] Author the dedicated family base YAML needed by:
      `taylor-and-francis-council-of-science-editors-author-date`,
      `taylor-and-francis-national-library-of-medicine`, and
      `springer-vancouver-brackets`.
- [ ] Re-run shared-fixture comparisons against the authored family bases.
- [ ] Convert public profiles to `extends:` wrappers only after parity is
      proven on the accepted verification surface.

## Acceptance

- standards-backed family base(s) exist where the authority chain requires them
- public publisher profiles no longer need to impersonate standards roots
- wrapper reductions are evidence-stable before landing

## Related

- csl26-nrkn
- csl26-wp6y
