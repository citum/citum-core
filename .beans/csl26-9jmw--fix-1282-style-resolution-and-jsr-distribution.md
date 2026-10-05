---
# csl26-9jmw
title: 'Fix #1282 style resolution and JSR distribution'
status: in-progress
type: bug
priority: high
tags:
    - bindings
    - wasm
    - styles
    - release
created_at: 2026-10-04T20:54:23Z
updated_at: 2026-10-05T13:42:25Z
---

Fix GitHub issue #1282 across the WASM bindings, style/version distribution, registry, locales, documentation, and release smoke tests.

Related specifications: docs/specs/WASM_SUPPORT.md, docs/specs/DISTRIBUTED_RESOLVER.md, docs/specs/FORWARD_COMPATIBILITY.md

- [x] Add shared binding style resolution and strict render-surface errors
- [x] Bundle MHRA and en-GB locale with locale-aware rendering
- [x] Restore legacy substitution compatibility and version diagnostics
- [x] Pin registry sources and synchronize shipped style versions
- [x] Update documentation and release smoke tests
- [x] Regenerate artifacts and pass all quality gates

Implementation complete on codex/fix-issue-1282. Verified: just pre-commit (2,822 tests), all 122 pinned catalog styles plus 20 built-ins, 351 JavaScript tests, 41 Python workflow tests, schema generation, Rust review-smell audit, and rebuilt staged JSR smoke tests under Node, Deno, and Chromium. The bean remains in progress only for the post-merge automated release, clean install of the exact published JSR version, and GitHub #1282 closure.
