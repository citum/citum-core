---
# csl26-2cvk
title: 'Promote citum-bindings: WASM API + specta type bindings'
status: completed
type: feature
priority: high
created_at: 2026-03-24T20:14:15Z
updated_at: 2026-03-24T23:20:26Z
---

Two related promotions in one PR batch:

1. **WASM API promotion (citum-core → citum-hub parity):**
   - Add HTML output to render_citation/render_bibliography (mode override, ensure_style_has_templates)
   - Add get_style_metadata, materialize_style (no intent-engine dep)
   - Add parse_references with CSL-JSON legacy fallback
   - Extract ensure_style_has_templates to citum-bindings

2. **Multi-language specta type bindings:**
   - bindings feature on citum-schema-data + citum-schema-style (specta::Type derive)
   - typescript feature on citum-bindings (specta-typescript exporter + CLI subcommand)
   - Design extensible to Swift/Kotlin/Go via same annotation path

Spec: docs/specs/LANGUAGE_BINDINGS.md

Cross-repo: citum-hub wasm-bridge slim (Steps 4-6) tracked as child work — slim to only 3 intent-engine functions after core PR merges.

## Progress

- [x] Phase 1: specta bindings on schema crates
- [x] Phase 2: citum-bindings WASM API promotion
- [x] Phase 3: spec activated
- [ ] citum-hub wasm-bridge slimming (separate PR)

## Summary of Changes

- Added bindings feature to citum-schema-data and citum-schema-style with specta::Type derives
- Promoted citum-bindings to stable WASM API (render_citation, render_bibliography, get_style_metadata, materialize_style, validate_style, ensure_style_has_templates)
- Implemented export_typescript() via specta TypeCollection + citum bindings --out-dir CLI subcommand
- Activated LANGUAGE_BINDINGS.md spec with full feature contract and output path documentation
- Merged as PR #428 on feat/bindings-promotion
