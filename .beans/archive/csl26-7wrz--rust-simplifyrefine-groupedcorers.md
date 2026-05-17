---
# csl26-7wrz
title: 'Rust simplify+refine: grouped/core.rs'
status: scrapped
type: task
priority: normal
created_at: 2026-05-17T00:42:46Z
updated_at: 2026-05-17T00:48:03Z
---

Coupled simplify + refine pass on `crates/citum-engine/src/processor/rendering/grouped/core.rs` (1708 lines, 5 `clippy::too_many_arguments` suppressions, 8+ functions ≥40 lines).

Plan: `~/.claude/plans/look-for-opportunities-for-modular-bubble.md`.

## Todo

- [ ] Commit 1: bundle params (GroupRenderParams + new TemplateRenderContext), remove 5 too_many_arguments allows
- [ ] Commit 2: extract template_policy.rs, component_predicates.rs, sentence_initial.rs
- [ ] Commit 3: split overlong functions in remaining core.rs
- [ ] Verify quality gate baseline + oracle no-diff on apa.csl
- [ ] Open PR refactor/grouped-core-simplify-refine

## Reasons for Scrapping

Duplicate of csl26-9f7g — first create call returned null but actually succeeded. Tracking moved to csl26-9f7g.
