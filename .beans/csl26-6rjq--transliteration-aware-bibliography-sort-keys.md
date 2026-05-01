---
# csl26-6rjq
title: Transliteration-aware bibliography sort keys
status: draft
type: feature
priority: normal
created_at: 2026-05-01T11:32:10Z
updated_at: 2026-05-01T11:45:00Z
---

Support romanization/transliteration-based sort keys so Arabic, Cyrillic, CJK names can optionally sort under their romanized forms. Currently explicitly out of scope in the Unicode sorting spec — deferred by design.

Users in some citation contexts (especially library and archival work) expect non-Latin names to sort under their romanized forms rather than original script order. This is a policy choice, not a collation bug, but will be reported as one.

Before implementing, three policy questions must be answered:
- Which transliteration standard to use (e.g. ALA-LC, ISO 233, Pinyin for CJK)?
- Is the sort key the original script, the displayed romanization, or a hidden romanized key invisible to the reader?
- Is transliteration applied globally or only for specific scripts/locales?

These choices affect both the data model and the user-visible bibliography output, so they warrant a spec before any implementation. Likely requires a new schema option (see csl26-xz2t) and possibly new reference data fields for pre-supplied romanized sort keys.
