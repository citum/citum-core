---
# csl26-jf8o
title: Replace serde_cbor with ciborium
status: completed
type: task
priority: normal
created_at: 2026-03-02T21:03:42Z
updated_at: 2026-03-02T21:09:12Z
---

serde_cbor is unmaintained. Replace with ciborium (Enarx-backed successor) across all Cargo.toml and call sites. 7 files, mechanical substitution.

## Summary of Changes\n\nReplaced serde_cbor (unmaintained) with ciborium 0.2 across 7 files. All call sites updated: from_reader over Cursor for deserialization, into_writer over Vec for serialization. All tests pass. Commit: 5c869d9
