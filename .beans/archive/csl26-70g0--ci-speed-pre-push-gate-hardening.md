---
# csl26-70g0
title: CI speed + pre-push gate hardening
status: completed
type: task
created_at: 2026-03-30T11:10:42Z
updated_at: 2026-03-30T12:15:00Z
---

Parallel lint+test jobs in ci.yml, nextest swap, fetch-depth:1, pre-push
Rust gate (fmt/clippy/nextest on .rs/.toml changes), Co-Authored-By rejection.
