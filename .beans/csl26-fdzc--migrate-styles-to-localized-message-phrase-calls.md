---
# csl26-fdzc
title: Migrate styles to localized message phrase calls
status: in-progress
type: task
priority: high
tags:
    - styles
    - localization
    - mf2
created_at: 2026-06-25T00:22:34Z
updated_at: 2026-06-26T11:40:06Z
---

## Problem

Styles still use English-centric template phrasing and the compatibility
`term:` component in places where locale-authored `message:` phrase calls
should own natural-language realization.

## Scope

Migrate checked-in `styles/` to the new localized message format in batches.
Start with embedded styles and embedded-locale phrase IDs, then continue
through the rest of `styles/` by style family or shared template pattern.

## Initial Batch

- Convert embedded styles from phrase-like `term:` or literal glue to
  `message:` calls where the locale should control word order.
- Prioritize `pattern.accessed-date`, `pattern.in-container`,
  `pattern.available-at`, and `pattern.retrieved-from`.
- Keep lexical and inflectional labels as `term.*` or `role.*` locale messages
  where they are labels rather than phrase realization.

## Acceptance Criteria

- Each batch preserves existing fidelity gates for the affected styles.
- Deprecated template `term:` use decreases monotonically across `styles/`.
- Missing message IDs and missing message args are caught by lint before merge.
- Embedded styles move through explicit proof and completion batches before
  broad non-embedded migration begins.
- Any new phrase IDs are documented in `docs/specs/LOCALE_MESSAGES.md`.


## Embedded Proof Batch (PR #965)

- Started the embedded migration with output-equivalent `message:` calls in
  representative embedded-core styles. This is a proof batch, not completion of
  all embedded styles.
- Converted representative `pattern.accessed-date` call sites in AMA, Chicago,
  Elsevier, MLA, and Springer Vancouver styles.
- Converted representative `pattern.in-container` call sites in Chicago, IEEE,
  and Springer author-date styles.
- Added grouped message arguments so `pattern.in-container` can receive an
  already-rendered container cluster such as editor plus parent-monograph title.
- Directly touched 9 embedded files. Wrapper styles may inherit those changes,
  but remaining embedded-core files still need classification and migration.
- Remaining embedded candidates include `elsevier-with-titles-core`,
  `springer-basic-brackets-core`,
  `taylor-and-francis-council-of-science-editors-author-date-core`,
  `taylor-and-francis-national-library-of-medicine-core`, and residual
  `term: in` sites in already-touched families.
- Deferred colon-bearing `in:` sites, `URL ` labels, and role-plus-name phrases
  until later batches define phrase IDs that preserve those outputs without
  encoding English glue inside arguments. (APA's container-author site and
  Chicago author-date's German-override container site were initially deferred
  here but were completed in the follow-up below.)

## Next Embedded Batch

- Classify each remaining embedded candidate by behavior, not namespace:
  lexical or inflectional label, phrase over rendered values, or contributor
  role/name phrase.
- Convert only output-preserving phrase-over-rendered-values sites to
  `message: pattern.*`.
- Keep role-label and role-plus-name migration out of this bean's next style
  batch unless contributor rendering semantics are explicitly revised first.
- Re-run parse/lint for all embedded styles and oracle/fidelity checks for each
  affected embedded family before marking embedded migration complete.

## APA And Chicago Follow-up (PR #965)

- Converted APA's chapter container-author phrase to
  `message: pattern.in-container` after fixing the hardcoded `Locale::en_us()`
  boundary so `Processor::new` can resolve default phrase messages.
- Converted Chicago author-date's chapter container phrase to
  `message: pattern.in-container`.
- Added `de-DE-chicago` override support for `pattern.in-container:
  "{$container}"` so the Chicago German variant preserves its intentional
  adjacency form without the English "In" phrase.
- Added regression coverage for message arguments over embedded and legacy
  container-author/title groups.
- Refreshed the top-10 oracle baseline after confirming the current aggregate
  has `cell` at 45/47 bibliography entries and no oracle regressions.

## PR #966 Follow-up Documentation

- PR #966 completes the checked-in template `term:` migration without taking on
  richer contributor phrase semantics. The immediate invariant remains:
  phrase localization uses `message: pattern.*`; `message: term.*` is reserved
  for atomic lexical or inflectional labels; contributor `label.term` is not the
  deprecated rendered template `term:` component.
- Rich locale message bodies are deferred. Current locale messages are plain
  text plus placeholders, and rich formatting should come from the template
  components that render message arguments. Deferred rich message bodies include
  both rich placeholders and rich locale-owned literal text, such as an
  italicized `In` inside a locale phrase. A future message pipeline should
  return format-neutral inline fragments before MF2 markup elements or any
  constrained inline message-body markup is enabled.
- Contributor-plus-role phrase realization is deferred. AMA-style `In:`
  editor/title phrasing and APA-style container contributor/title phrasing need
  future `pattern.*` messages with a designed argument shape; exact message IDs
  are intentionally not reserved in this batch.
- The durable design boundary is documented in
  `docs/specs/LOCALE_MESSAGES.md`, with a cross-reference from
  `docs/specs/DJOT_RICH_TEXT.md`.
