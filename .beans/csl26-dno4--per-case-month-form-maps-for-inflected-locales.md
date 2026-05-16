---
# csl26-dno4
title: Per-case month-form maps for inflected locales
status: draft
type: feature
priority: low
tags:
    - dates
    - multilingual
created_at: 2026-05-16T11:28:30Z
updated_at: 2026-05-16T12:48:56Z
---

## Goal

Allow a locale to store multiple inflected forms of each month name (e.g. nominative, genitive, locative) under `dates.months.<case>` keys, and let `pattern.date-*` messages dispatch on case via a `:form` selector. Defer until a real locale actually needs it.

## Status

**Draft** — speculative; do not implement until a concrete locale need surfaces.

## When this matters

`csl26-v6ok` works for Basque because every month lemma ends in `-a`, so a single citation-form month list plus an `-(r)en` suffix in the pattern covers the bibliography full-date case. That trick stops working when:

- The locale needs the same month name in two different cases within one render (e.g. genitive in the full date AND locative in an accessed-date phrase).
- The morphological suffix is not uniform across month names (locales where some months end in `-a`, some in `-ua`, etc., and need different inflected forms rather than a uniform suffix).

If either condition fires for a locale that we want to ship, this bean becomes blocking.

## Design sketch

```yaml
# Hypothetical multi-form month list
dates:
  months:
    long:
      nominative: [urtarrila, otsaila, …]
      genitive:   [urtarrilaren, otsailaren, …]
      locative:   [urtarrilan, otsailan, …]
```

Pattern selector:

```yaml
messages:
  pattern.date-full: |
    .match {$month_form :select}
    when genitive {{$year}ko {$month} {$day}a}
    when * {{$month} {$day}, {$year}}
```

## Todo

- [ ] Wait for a concrete locale need
- [ ] Design schema additions (back-compat with the current `Vec<String>` shape)
- [ ] Wire `:form` selector + `month_form` into `MessageArgs`
- [ ] Migrate `eu-ES` if Basque review confirms two forms are needed

## Related

- Parent feature: `csl26-v6ok`
- Spec: `docs/specs/LOCALE_MESSAGES.md` §1.5 (already calls this out as a follow-up)
