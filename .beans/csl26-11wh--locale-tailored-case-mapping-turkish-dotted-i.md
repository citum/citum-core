---
# csl26-11wh
title: Locale-tailored case mapping (Turkish dotted i)
status: todo
type: bug
priority: normal
tags:
    - multilingual
    - engine
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-19T11:05:15Z
parent: csl26-0ugp
---

Case transforms in crates/citum-engine/src/values/text_case.rs use Rust's locale-blind to_uppercase()/to_lowercase(), so Turkish and Azerbaijani i/İ and ı/I map incorrectly under uppercase/lowercase/sentence transforms. The sting: Citum's own embedded tr-TR locale file is among the most complete in the tree (schema-v2, full MF2 coverage) — locale-data investment offers no protection against this engine-level casing gap. Adopt locale-tailored case mapping (ICU4X casemap) or explicitly gate the transforms with a diagnostic. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(e).
