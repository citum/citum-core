---
# csl26-cq35
title: 'Interactive API: per-document style overrides'
status: completed
type: feature
priority: normal
created_at: 2026-06-04T14:21:32Z
updated_at: 2026-06-04T18:23:49Z
---

The interactive server API (`format_document` + session `open_session`) accepts a `StyleInput` union (`id`/`uri`/`path`/`yaml`) but has no mechanism to merge a partial per-document override over a base style.

Word-processor hosts (citum-office) need per-document overrides: e.g. APA base style but `author_connector: &` for one document only, without editing the shared style.

## Scope

- Add a per-document override input to `FormatDocumentRequest` and `open_session` (e.g. `style_overrides`: partial style YAML/JSON merged over the resolved base style).
- Apply the merge after style resolution, before processing; scope to the request/session only.
- Define which fields are overrideable (connectors, et-al thresholds, particle handling, etc.).
- Tests: base style + override changes rendered output for that request only; base style untouched.

## Origin

Required by citum-office P2 "Document Overrides" dialog. Tracked gap in
citum-office docs/specs/01-cdip-protocol.md § Known Engine Gaps.

## Summary of Changes

- Added `Style::apply_overlay` public method in `citum-schema-style` — thin wrapper over the existing `merge_style_overlay` (used by `extends` inheritance), making overlay merge available to surface crates.
- Added `apply_style_overrides(style, overlay_src)` helper and `style_overrides: Option<String>` field to `FormatDocumentRequest` in `citum-engine`. The override is applied in `format_document_with_style` (the single choke point), covering all three entry points (`format_document`, `format_document_with_resolver`, `format_document_with_style`).
- Wired `style_overrides` into `citum-server` `OpenSessionParams` and the `open_session` handler (pre-merge before `DocumentSession::new`). Also added to schema-mirror `FormatDocumentParams`.
- `format_document` server path flows through automatically (deserializes `FormatDocumentRequest` directly).
- Tests: 3 engine unit tests (`apply_style_overrides_merges_option_field`, `style_overrides_invalid_yaml_returns_parse_error`, `style_overrides_and_symbol_changes_rendered_output`), 1 session test (`session_style_override_produces_divergent_output`), 2 server RPC tests (`format_document_style_overrides_changes_and_connector`, `open_session_style_overrides_changes_and_connector`).
- Spec at `docs/specs/INTERACTIVE_STYLE_OVERRIDES.md`.
- Schemas regenerated.
