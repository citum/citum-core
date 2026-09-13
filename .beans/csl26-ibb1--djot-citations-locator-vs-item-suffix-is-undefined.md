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
updated_at: 2026-09-13T17:03:01Z
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

## Correction (2026-09-13): the label/no-label split is already solved; narrow the real bug

Re-checked against pandoc's own convention: unlabeled locator text defaulting to `page` is not an
open design question. It's a standard, working rule (pandoc has always done this), and our code
already implements it identically — `parse_locator_segment`
(`crates/citum-schema-data/src/citation.rs:675`) falls back to `LocatorType::Page` when no label
matches. So `@foo, 33` and `@foo, p. 33` already resolve the same way, correctly, today.

The actual bug is narrower than originally scoped: `parse_locator_segment` applies that fallback
**unconditionally** — it does not check that the unlabeled text looks like a plausible locator
value (a number, range, roman numeral, etc.) before defaulting to page. So a genuine item suffix
with no locator label at all, e.g. jgm's own example `@baz, with all its quirks`, is silently
misparsed as `LocatorSegment::new(Page, "with all its quirks")` instead of being treated as suffix
text.

## Revised scope

- Add a shape check before the page-default fallback in `parse_locator_segment`: only default to
  `Page` when the remaining text plausibly looks like a locator value (numeric, numeric range,
  roman numeral); otherwise fall through to item-suffix handling (which doesn't exist yet — see
  csl26-esq8 / the affix-support gap).
- This is now a same-crate implementation bug plus a dependency on item-suffix support existing at
  all, not an open upstream design question. Nothing to raise on djot#32.
- Related, not duplicate: `docs/specs/LOCATOR_INPUT.md` already documents a different known
  false-positive in the plurality heuristic ("figure A-3"). That's the *shape* of a locator value
  being misread as plural; this bug is about text with *no locator shape at all* being forced into
  a locator. Cross-reference both when fixing.

## Origin

Caught while drafting a reply to jgm on djot#32; verifying the claim against `citation.rs` showed
pandoc's convention already covers what I thought was open, and surfaced the real, narrower bug.
