---
# csl26-j5rz
title: 'feat: style registry (alias + discovery layer)'
status: in-progress
type: feature
priority: normal
created_at: 2026-03-20T19:12:28Z
updated_at: 2026-03-21T00:51:05Z
---

Replace hardcoded EMBEDDED_STYLE_ALIASES/EMBEDDED_STYLE_NAMES slices with a serde-driven StyleRegistry type backed by a YAML registry file. Support three-layer resolution: local registry file, embedded default, filesystem fallback. Expose citum registry subcommands. Spec at docs/specs/STYLE_REGISTRY.md.

## Review fixes applied (2026-03-20)

- Fixed IEEE entry: removed self-alias, expanded description
- Wired registry/default.yaml via include_bytes! so descriptions populate at runtime
- Added load_from_file with builtin/path XOR validation
- Restored SchemaType ValueEnum for clap validation
- Fixed citation/citations key consistency
- Updated spec: --target → positional, citum styles not deprecated, correct paths
