---
# csl26-h1ms
title: 'Substitute editor label: comma + short form (eds.)'
status: completed
type: task
priority: normal
created_at: 2026-06-21T17:46:30Z
updated_at: 2026-06-21T18:34:09Z
---

Deferred from csl26-maim. Substitute editor-as-author renders ' (eds.)' (parens) for AMA/IEEE; guides want ', eds.' (comma + short). APA legitimately wants the parens form, MLA wants ', editors' (long), IEEE wants ', Eds.,' (capitalized). No existing RoleLabelPreset produces comma+short. Needs a new ShortSuffixComma preset variant + per-style YAML + text-case for IEEE capitalization. Root: crates/citum-engine/src/values/contributor/labels.rs resolve_role_label_preset (ShortSuffix hard-codes ' (eds.)' at the parens path); substitute path in crates/citum-engine/src/values/contributor/substitute.rs. Audit rows 86/156/172 in docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md.

## Summary of Changes

Added a comma-joined short role-label form for the editor-as-author
substitute path and a `text-case` channel so styles can capitalise the
substitute label.

- Schema: new `RoleLabelPreset::ShortSuffixComma` variant (serde
  `short-suffix-comma`; legacy form `short-comma`); new optional
  `Substitute.contributor-role-case` (`TextCase`) threaded through merge/default.
- Engine: `resolve_role_label_preset` renders `ShortSuffixComma` as `, <short>`
  and applies the optional `text-case` to the resolved term in the short/long/
  comma suffix arms (bundled gender+case into `RoleLabelTermOptions`); substitute
  path maps `short-comma` and passes `contributor-role-case`.
- Styles: IEEE (`short-comma` + `capitalize-first` → `, Eds.`), AMA
  (`short-comma` → `, eds.`), MLA (`long` → `, editors`). APA unchanged (`(eds.)`).
- Verified by direct render of an editor-only book across all four styles plus
  a new bibliography integration test; APA oracle unchanged (no regression).
  Audit rows 86/156/172 resolved.
