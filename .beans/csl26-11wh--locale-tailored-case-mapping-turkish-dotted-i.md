---
# csl26-11wh
title: Locale-tailored case mapping (Turkish dotted i)
status: todo
type: bug
tags:
    - multilingual
    - engine
created_at: 2026-07-18T20:32:33Z
updated_at: 2026-07-18T20:32:33Z
parent: csl26-0ugp
---

Case transforms in crates/citum-engine/src/values/text_case.rs use Rust's locale-blind to_uppercase()/to_lowercase(), so Turkish and Azerbaijani i/İ and ı/I map incorrectly under uppercase/lowercase/sentence transforms — ironic given tr-TR is one of the best-developed locales. Adopt locale-tailored case mapping (ICU4X casemap) or explicitly gate the transforms with a diagnostic. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(e).
