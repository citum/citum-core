---
# csl26-6rjq
title: Transliteration-aware bibliography sort keys
status: completed
type: feature
priority: normal
tags:
    - multilingual
created_at: 2026-05-01T11:32:10Z
updated_at: 2026-07-08T13:25:18Z
---

Support romanization/transliteration-based sort keys so Arabic, Cyrillic, CJK names can optionally sort under their romanized forms. Currently explicitly out of scope in the Unicode sorting spec — deferred by design.

Users in some citation contexts (especially library and archival work) expect non-Latin names to sort under their romanized forms rather than original script order. This is a policy choice, not a collation bug, but will be reported as one.

Before implementing, three policy questions must be answered:
- Which transliteration standard to use (e.g. ALA-LC, ISO 233, Pinyin for CJK)?
- Is the sort key the original script, the displayed romanization, or a hidden romanized key invisible to the reader?
- Is transliteration applied globally or only for specific scripts/locales?

These choices affect both the data model and the user-visible bibliography output, so they warrant a spec before any implementation. Likely requires a new schema option (see csl26-xz2t) and possibly new reference data fields for pre-supplied romanized sort keys.

## Design (signed off 2026-07-08; normative spec: docs/specs/MULTILINGUAL_SORTING.md)

Answered jointly with csl26-xz2t — the full recommended design (schema sketch, three policy answers, phasing) is recorded there. Summary of this bean's part: primary mechanism is data-supplied `sort_as` keys on contributor names and titles (biblatex sortname/sorttitle prior art), hidden from rendered output, activated per-script via `options.sorting.multilingual: romanized`; generated transliteration deferred to a feature-gated phase 3 (follow-up bean).

**Sign-off (2026-07-08):** the three policy questions are answered canonically in docs/specs/MULTILINGUAL_SORTING.md §Design 4: (1) no global standard — data-supplied keys primary, generated transliteration is feature-gated Phase 3; (2) hidden romanized sort key, rendering unchanged; (3) per-script/per-locale only, never a global transform. Romanized-mode fallback is the three-step chain: explicit sort-as → §1.3-matched transliteration → UCA on original text.

## Todo

- [x] Spec revisions signed off by Bruce (shared with csl26-xz2t)
- [x] `sort-as` on MultilingualComplex (titles + name parts) and MultilingualName (holistic, wins over part-level), skip-serialized, documented
- [x] Engine consumes sort-as/transliterations under `multilingual: romanized` via the three-step chain
- [x] Tests: ALA-LC Cyrillic archival case, transliteration-map fallback, neither-key fallback, sort-as never rendered, mixed-script fixtures
- [x] Fidelity guard recorded: report-core no-op comparison attempted 2026-07-08; blocked by existing oracle failures (`Total styles with errors: 39`), while targeted default/uniform no-op tests and `just pre-commit` pass
