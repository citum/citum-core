---
# csl26-j5rz
title: 'feat: style registry (alias + discovery layer)'
status: completed
type: feature
priority: normal
created_at: 2026-03-20T19:12:28Z
updated_at: 2026-03-21T11:28:07Z
---

Replace hardcoded EMBEDDED_STYLE_ALIASES/EMBEDDED_STYLE_NAMES slices with a serde-driven StyleRegistry type backed by a YAML registry file. Support three-layer resolution: local registry file, embedded default, filesystem fallback. Expose citum registry subcommands. Spec at docs/specs/STYLE_REGISTRY.md.

## Review fixes applied (2026-03-20)

- Fixed IEEE entry: removed self-alias, expanded description
- Wired registry/default.yaml via include_bytes! so descriptions populate at runtime
- Added load_from_file with builtin/path XOR validation
- Restored SchemaType ValueEnum for clap validation
- Fixed citation/citations key consistency
- Updated spec: --target → positional, citum styles not deprecated, correct paths

## Summary of Changes

Introduced `StyleRegistry` and `RegistryEntry` serde/schemars types in `citum-schema-style`, backed by `registry/default.yaml` embedded via `include_bytes!`. Implemented three-layer resolution: local `citum-registry.yaml` → embedded default → filesystem `styles/<name>.yaml`. Added `load_from_file` with builtin/path XOR validation and alias resolution. Exposed `citum registry list` and `citum registry resolve` subcommands. Added `registry.json` to `docs/schemas/`. Spec: `docs/specs/STYLE_REGISTRY.md`. Landed in feat(schema): add StyleRegistry type, registry, and CLI (a895272).
