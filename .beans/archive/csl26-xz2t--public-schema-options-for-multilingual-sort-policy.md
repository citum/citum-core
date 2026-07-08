---
# csl26-xz2t
title: Public schema options for multilingual sort policy
status: completed
type: feature
priority: normal
tags:
    - multilingual
created_at: 2026-05-01T11:32:12Z
updated_at: 2026-07-08T13:22:11Z
---

Expose schema/style configuration for multilingual bibliography sort behavior once multiple modes exist (e.g. single-locale, per-script, transliterated). Currently all sort policy is hardcoded. Depends on per-script partitioning and/or transliteration features being implemented first. Deferred by design per UNICODE_BIBLIOGRAPHY_SORTING.md.

## Design (joint with csl26-6rjq — signed off 2026-07-08; normative spec: docs/specs/MULTILINGUAL_SORTING.md)

Decide the two beans together; csl26-6rjq's data-model answer determines this bean's schema surface.

Proposed schema (options.sorting, style-level with bibliography-level override):

    sorting:
      locale: auto | <bcp47>          # collator locale; default auto = effective bibliography locale
      multilingual: uniform | romanized | per-script
      # uniform    = today's single-collator pass (default, no behavior change)
      # romanized  = sort under supplied/generated romanized keys, render original script
      # per-script = partition by script, order partitions per locale convention (later phase)

Recommended answers to csl26-6rjq's three policy questions:

1. **Standard:** do not pick one globally. Primary mechanism is **data-supplied sort keys** (biblatex prior art: sortname/sortkey/sorttitle): add optional `sort_as` to contributor names and titles in the reference schema. Generated transliteration (ALA-LC for Cyrillic, Pinyin for Han) is a later, feature-gated fallback only where a deterministic Rust implementation exists.
2. **Visibility:** hidden sort key — sort under the romanized form, render the original script unchanged. Matches library/archival expectation and biblatex behavior.
3. **Scope:** per-script via the `multilingual` mode, never a global text transform.

**Phasing (unblocks archival users without a transliteration engine):**
- Phase 1: reference-data `sort-as` fields + `options.sorting` with `uniform`/`romanized`. Under `romanized`, keys resolve via the three-step chain (explicit `sort-as` → §1.3-matched transliteration → UCA on original text; see spec §Design 3). Small, self-contained.
- Phase 2: `per-script` partition mode (partition order = locale convention, spec addendum needed).
- Phase 3: generated transliteration behind a cargo feature.

Schema change ⇒ `just schema-gen` in the implementing commit; spec doc in docs/specs/ superseding the out-of-scope declarations in UNICODE_BIBLIOGRAPHY_SORTING.md §Scope.

**Left in draft deliberately:** the three policy answers above are recommendations; promote to todo after Bruce confirms them.

**Sign-off (2026-07-08):** Bruce confirmed the three policy answers and chose: (a) this PR carries Phases 1+2, Phase 3 gets a follow-up bean; (b) romanized-mode fallback is the three-step chain (sort-as → matched transliteration → UCA on original). Normative spec: docs/specs/MULTILINGUAL_SORTING.md.

## Todo

- [x] Spec commit: docs/specs/MULTILINGUAL_SORTING.md (Draft) + scope edits to UNICODE_BIBLIOGRAPHY_SORTING.md, SORTING.md, MULTILINGUAL.md §4.1, MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md — spec design approved by Bruce 2026-07-08
- [x] Phase 1 schema: SortingConfig (locale, multilingual) on Config + BibliographyOptions override, merge machinery, roundtrip tests
- [x] Phase 1 engine: three-step romanized sort-key chain in sort_support.rs / Sorter paths
- [x] just schema-gen in the implementing commit; spec Draft → Active
- [x] Phase 2: per-script shorthand expands to sort-partitioning {by: script, mode: sort-only} when absent; explicit block authoritative; precedence tests
- [x] Create follow-up bean for Phase 3 (feature-gated transliteration registry): csl26-rxik

## Summary of Changes

Implemented Phases 1–2 from docs/specs/MULTILINGUAL_SORTING.md: added options.sorting with style/bibliography merge behavior, wired romanized sort-key policy through bibliography sorting, implemented per-script shorthand precedence, regenerated schemas, and created Phase 3 follow-up bean csl26-rxik.
