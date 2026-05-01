# Unicode Bibliography Sorting Specification

**Status:** Active
**Date:** 2026-04-16
**Related:** unicode-aware bibliography ordering regression

## Purpose

Define locale-aware bibliography sorting for Citum so accented and other non-ASCII names sort according to Unicode collation rules instead of raw bytewise string order.

## Scope

In scope: bibliography author/title sort comparisons in the engine, shared sort-key normalization, regression fixtures, examples, and tests. Out of scope: new public schema options, transliteration-aware sorting, per-script partitioning, or broader multilingual rendering redesign.

## Collation Policy

Citum sorts bibliography strings using a locale-tailored collator derived from the effective bibliography locale (for example, `en-US`, `ar`, `ko`). The collator is based on the Unicode Collation Algorithm (UCA) as tailored by the Common Locale Data Repository (CLDR).

**Fallback chain:** When the requested locale identifier cannot be parsed or has no tailored collation data, Citum attempts to find a valid locale by progressively removing locale subtags (for example, `de-DE-foo_bar` → `de-DE` → `de`). If no subtag-reduced variant is recognized, Citum falls back to `en-US` as the final default. The `en-US` fallback is a consistency guarantee — it produces a stable, reproducible order across all systems — but it is NOT linguistically correct for scripts that have their own tailored ordering rules (Arabic, Hangul, Han characters, etc.). Domain experts and maintainers should understand this limitation: a bibliography sorted under a fallback collator for an unsupported language will not sort the same way a native speaker would expect.

**Single pass:** One collator is used for the entire bibliography in a single sort pass. Per-script partitioning (e.g., sorting Latin names separately from Arabic names) is explicitly out of scope. Transliteration-based sorting (e.g., Romanizing Arabic for ASCII-only systems) is explicitly out of scope.

## Collation Options

Citum applies the following configuration to the Unicode Collator (defined in plain language for domain experts, not as code):

| Option | Value | Semantics |
|--------|-------|-----------|
| **Strength** | Secondary | Compare base letters (primary level) and accents/diacritics (secondary level). Case differences are NOT distinguished at the primary or secondary levels. |
| **Case Level** | Off | Case-insensitive sorting is achieved via collator configuration, not by pre-processing (lowercasing) source text. The original text is passed to the collator unchanged. |
| **Alternate Handling** | Shifted | Punctuation and whitespace are treated as ignorable at the primary and secondary levels. Leading particles (e.g. "al-" or "O'") and other punctuation in names do not break alphabetical ordering. |
| **Normalization** | ICU default (not explicitly configurable) | Unicode normalization is handled internally during collation. Strings that are canonically equivalent (e.g. `"é"` as a single codepoint vs `"e" + acute-accent`) sort equal. |
| **Numeric Ordering** | Off (disabled) | Numbers are compared as character sequences, not as numeric values. For example, "Item 2" sorts before "Item 10" as strings, not numerically. |
| **Script Reordering** | ICU default (not explicitly configurable) | No custom script reordering is applied at the Citum API level. Mixed-script bibliographies use the default CLDR script ordering for the resolved locale. |

## Deterministic Tie-Breaking

When two entries compare equal under all collation comparisons (same author, same title, same date, etc.), a deterministic tiebreaker is applied:

1. Apply the configured sort key chain from the style's sort specification (typically author → year → title; the current sort key chain is available in the processor's SortKey enum; additional fields may be added in future versions).
2. If still equal after all sort fields, compare entry identifiers (citation keys) as plain strings.
3. The sort algorithm is stable: entries that are collator-equal through all steps retain their original input order if the entry identifiers are also equal.

This ensures reproducible, deterministic results in all scenarios.

## Acceptance Criteria

- [ ] Author/date bibliography sorting places accented surnames near their ASCII peers instead of at the end of the list.
- [ ] Grouped bibliography sorting and top-level bibliography sorting use the same locale-aware text comparison path.
- [ ] Sorting is case-insensitive: "Smith" and "smith" sort as equal at the primary comparison level.
- [ ] Sorting is case-insensitive without lowercasing: original text is preserved and passed to the collator unchanged.
- [ ] Punctuation and whitespace in names (e.g. leading "al-" or "O'") do not break alphabetical ordering.
- [ ] Two entries that are collator-equal through all sort fields are ordered deterministically by entry identifier, not insertion order.
- [ ] Root collation is used as fallback when no tailored locale data exists.
- [ ] Regression fixtures include accented surnames, NFC/NFD equivalents, and a mixed-case tiebreaker case.

## Changelog

- 2026-04-16: Initial version.
- 2026-05-01: Expand spec with domain-expert-friendly Collation Policy, option table, and tie-breaking semantics; clarify fallback guarantees and limitations.
