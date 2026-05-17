---
# csl26-9f7g
title: Rust simplify+refine pass on engine grouped/core.rs
status: in-progress
type: task
priority: normal
created_at: 2026-05-17T00:42:49Z
updated_at: 2026-05-17T07:49:10Z
---

Coupled simplify + refine pass on grouped/core.rs (1708 lines, 5 clippy::too_many_arguments suppressions, 8+ overlong fns). Plan: ~/.claude/plans/look-for-opportunities-for-modular-bubble.md

## Todo

- [x] Commit 1: bundle params (GroupRenderParams + new TemplateRenderContext), remove 5 too_many_arguments allows
- [x] Commit 2: extract template_policy.rs, component_predicates.rs, sentence_initial.rs
- [x] Commit 3: split overlong functions in remaining core.rs
- [x] Verify quality-gate baseline + oracle no-diff on apa.csl (154 styles fidelity=1.0, apa.csl 51/51 match)
- [x] Open PR refactor/grouped-core-simplify-refine — #731 https://github.com/citum/citum-core/pull/731
