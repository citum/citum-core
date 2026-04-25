---
# csl26-jk10
title: Evaluate CBOR embedding for builtin styles
status: todo
type: task
priority: normal
tags:
    - styles
    - performance
    - build
    - infra
    - research
created_at: 2026-04-21T17:10:10Z
updated_at: 2026-04-25T20:20:07Z
---

Assess converting embedded builtin style assets from YAML source files to build-generated CBOR blobs for runtime embedding.

Questions to answer:
- Measure binary size and startup/deserialization impact for builtin style loading.
- Decide whether YAML-preserved validation semantics need a parallel source-of-truth or build-time validation step.
- Recommend keep-YAML, dual YAML+CBOR, or CBOR-only embedding for builtins.

Context: introduced while implementing docs/specs/CONFIG_ONLY_PROFILE_OVERRIDES.md on branch codex/implement-config-only-profile-overrides.
