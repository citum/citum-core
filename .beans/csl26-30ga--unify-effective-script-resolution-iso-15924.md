---
# csl26-30ga
title: Unify effective-script resolution (ISO 15924)
status: todo
type: task
tags:
    - multilingual
    - engine
created_at: 2026-07-18T20:31:57Z
updated_at: 2026-07-18T20:31:57Z
parent: csl26-0ugp
---

Replace the boolean is_latin_script_language classifier (crates/citum-engine/src/values/mod.rs) with a single resolver that returns an ISO 15924 script code for an item/field's effective language, preserving the positive-evidence rule (absent or unrecognized evidence resolves to no script, never a default). Keep a thin bool adapter so existing call sites are unchanged. This unblocks script-keyed punctuation realization and any future per-script behavior beyond the current latin/not-latin split. See docs/architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md §2(b).
