---
# csl26-ibb1
title: 'Djot citations: locator vs item-suffix is undefined'
status: todo
type: bug
priority: normal
tags:
    - citation
    - parser
    - engine
created_at: 2026-09-12T13:42:45Z
updated_at: 2026-09-12T13:42:45Z
parent: csl26-s2zo
---

`parse_citation_item_no_integral` treats everything after the first comma in a citation item as a
locator and hands it to `normalize_locator_text`. But jgm's djot#32 grammar gives that same position
two distinct meanings:

```
[@foo, p. 33; @baz, with all its quirks]
       ^locator          ^item suffix
```

The two are syntactically identical. Our parser has no rule distinguishing them, and the upstream
spec does not define one either — jgm listed "how are locators distinguished from other suffix
content?" as an open question in the thread.

Current behaviour when no locale locator label matches is undefined-by-accident: the text still
becomes a locator. That means `@baz, with all its quirks` silently produces a garbage locator.

## Scope

- Decide the discriminator. Localized label detection is the natural one: if the segment opens with
  a known locator term (`p.`, `ch.`, `sec.`, …) it is a locator, otherwise it is an item suffix.
- Decide the no-label case explicitly — bare `@foo, 33` (number, no label) is the ambiguous edge.
- Requires item-suffix support to exist first (see csl26-esq8), so this is a design decision to
  settle alongside it, not after.
- Tests: a labelled locator, an unlabelled numeric locator, and a prose suffix in one cluster.

## Origin

Surfaced while reviewing jgm's restated proposal on https://github.com/jgm/djot/issues/32 against
our parser. Worth raising upstream as well as resolving here.
