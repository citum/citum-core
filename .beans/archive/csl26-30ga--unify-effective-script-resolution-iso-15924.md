---
# csl26-30ga
title: Unify effective-script resolution (ISO 15924)
status: completed
type: task
priority: normal
tags:
    - multilingual
    - engine
created_at: 2026-07-18T20:31:57Z
updated_at: 2026-07-19T12:28:51Z
parent: csl26-0ugp
---

Replace the boolean is_latin_script_language classifier (crates/citum-engine/src/values/mod.rs) with a single resolver that returns an ISO 15924 script code for an item/field's effective language, preserving the positive-evidence rule (absent or unrecognized evidence resolves to no script, never a default). Keep a thin bool adapter so existing call sites are unchanged. This unblocks script-keyed punctuation realization and any future per-script behavior beyond the current latin/not-latin split. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(b).

## Summary of Changes

Replaced the Latin/non-Latin classifier with an ICU4X likely-subtags resolver
that returns canonical ISO 15924 codes only from positive BCP 47 evidence. Kept
the existing Latin boolean adapter, added explicit/inferred/unknown-language
coverage, and verified no-default-feature and punctuation compatibility.
