---
# csl26-g49a
title: Implement NumberForm::Ordinal in the number-rendering engine
status: todo
type: feature
priority: low
tags:
    - engine
    - style
    - multilingual
    - fidelity
created_at: 2026-07-17T10:15:37Z
updated_at: 2026-07-17T11:01:32Z
---

TemplateNumber.form: Option<NumberForm> already has an Ordinal variant in the schema, but crates/citum-engine/src/values/number.rs never reads self.form at all -- NumberForm::Ordinal is a schema-only no-op today. GB/T 7714's CSL-M source wants ordinal number form (5th, 2nd) for numeric editions in English-language references (see gbt7714.7.4:5, 8.2.2:6, 8.3.2:4, 8.3.2:5 in the upstream corpus -- citum currently renders bare '5'/'4'/'6' instead of '5th'/'4th'/'6th', the only remaining diff against oracle for those 4 entries; both sides otherwise agree, including the 'editor(s)' role-label text). Needs locale-aware ordinal suffix rules (English 1st/2nd/3rd/4th... at minimum; do not apply English-style suffixes to zh-CN numeric editions, which use a bare numeral + suffix term instead, per TEMPLATE_V3.md §2.4).
