---
# csl26-xz2t
title: Public schema options for multilingual sort policy
status: draft
type: feature
priority: normal
tags:
    - multilingual
created_at: 2026-05-01T11:32:12Z
updated_at: 2026-07-06T18:47:52Z
---

Expose schema/style configuration for multilingual bibliography sort behavior once multiple modes exist (e.g. single-locale, per-script, transliterated). Currently all sort policy is hardcoded. Depends on per-script partitioning and/or transliteration features being implemented first. Deferred by design per UNICODE_BIBLIOGRAPHY_SORTING.md.

## Recommended Design (2026-07-06, joint with csl26-6rjq — needs policy sign-off)

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
- Phase 1: reference-data `sort_as` fields + engine consumes them when present under `multilingual: romanized`; absent keys fall back to UCA collation (UNICODE_BIBLIOGRAPHY_SORTING.md unchanged). Small, self-contained.
- Phase 2: `per-script` partition mode (partition order = locale convention, spec addendum needed).
- Phase 3: generated transliteration behind a cargo feature.

Schema change ⇒ `just schema-gen` in the implementing commit; spec doc in docs/specs/ superseding the out-of-scope declarations in UNICODE_BIBLIOGRAPHY_SORTING.md §Scope.

**Left in draft deliberately:** the three policy answers above are recommendations; promote to todo after Bruce confirms them.
