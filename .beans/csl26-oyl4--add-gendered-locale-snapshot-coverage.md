---
# csl26-oyl4
title: Add gendered locale snapshot coverage
status: todo
type: task
priority: low
tags:
    - locale
    - testing
    - schema
created_at: 2026-04-29T15:43:17Z
updated_at: 2026-04-29T15:44:22Z
parent: csl26-li63
---

Follow-up split from csl26-y3kj after the MaybeGendered<T> locale schema work landed. Add focused snapshot coverage for gendered locale rendering so the completed implementation has durable regression fixtures.

## Tasks

- [ ] Add a French snapshot test for a gendered editor role label.
- [ ] Add an Arabic snapshot test for a gendered ordinal or locator term.
- [ ] Confirm existing plain-string locale fixtures still render unchanged.

## Context

The model and runtime work landed in csl26-y3kj. This bean tracks only the remaining snapshot coverage gap.
