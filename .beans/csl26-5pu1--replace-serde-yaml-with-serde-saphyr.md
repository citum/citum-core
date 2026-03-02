---
# csl26-5pu1
title: Replace serde_yaml with serde-saphyr
status: completed
type: task
priority: normal
created_at: 2026-03-02T21:19:23Z
updated_at: 2026-03-02T21:51:03Z
---

serde_yaml 0.9 is deprecated with RUSTSEC-2024-0320 (ACE via unsafe tag resolution). serde-saphyr is the maintained successor with identical API. Use Cargo package alias to avoid touching .rs files.

## Summary of Changes\n\nReplaced serde_yaml with serde-saphyr 0.0.21 on branch deps/yaml-serde-saphyr. Required fixing Value/from_value usages in io.rs and djot.rs to use direct typed deserialization. 474/474 tests pass. Commit: d04c69b
