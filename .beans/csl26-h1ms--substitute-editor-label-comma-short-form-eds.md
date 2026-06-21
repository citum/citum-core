---
# csl26-h1ms
title: 'Substitute editor label: comma + short form (eds.)'
status: todo
type: task
priority: normal
created_at: 2026-06-21T17:46:30Z
updated_at: 2026-06-21T17:46:30Z
---

Deferred from csl26-maim. Substitute editor-as-author renders ' (eds.)' (parens) for AMA/IEEE; guides want ', eds.' (comma + short). APA legitimately wants the parens form, MLA wants ', editors' (long), IEEE wants ', Eds.,' (capitalized). No existing RoleLabelPreset produces comma+short. Needs a new ShortSuffixComma preset variant + per-style YAML + text-case for IEEE capitalization. Root: crates/citum-engine/src/values/contributor/labels.rs resolve_role_label_preset (ShortSuffix hard-codes ' (eds.)' at the parens path); substitute path in crates/citum-engine/src/values/contributor/substitute.rs. Audit rows 86/156/172 in docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md.
