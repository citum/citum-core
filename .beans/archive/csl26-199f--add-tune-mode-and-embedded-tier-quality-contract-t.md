---
# csl26-199f
title: Add tune mode and embedded-tier quality contract to style-evolve skills
status: completed
type: feature
priority: normal
created_at: 2026-06-24T14:50:59Z
updated_at: 2026-06-24T15:00:56Z
---

Refactor style-evolve skill family to reflect the LLM hand-tuning era: add a new 'tune' mode for driving embedded-core styles to 100% fidelity + clean SQI, establish embedded-tier portfolio concept, make SQI a co-primary gate for embedded styles, reposition citum-migrate as evidence/seed. Changes span shared docs, both .skills/ and .claude/skills/ mirrors.

## Todo

- [x] Create branch feat/style-evolve-tune-mode
- [x] Update docs/policies/STYLE_WORKFLOW_DECISION_RULES.md (tier axis, SQI gate)
- [x] Update docs/guides/STYLE_WORKFLOW_EXECUTION.md (tier-aware verification, tune loop)
- [x] Update .skills/style-evolve/SKILL.md (tune mode)
- [x] Update .claude/skills/style-evolve/SKILL.md (tune mode, routing)
- [x] Create .claude/skills/style-tune/SKILL.md (new hand-tuning sub-skill)
- [x] Update .claude/skills/style-qa/SKILL.md (tier-aware gates)
- [x] Update .claude/skills/style-migrate-enhance/SKILL.md (migrate-as-seed framing)
- [x] Update docs/guides/AGENT_SKILLS.md (tune mode table)
- [x] Update docs/architecture/SKILL_AGENT_REFACTOR.md (topology note)
- [x] Verify docs/beans hygiene check passes
- [x] Commit + PR

## Summary of Changes

Added a fourth public mode 'tune' to the style-evolve skill family. Established
the embedded-core portfolio tier (16 binary-embedded styles) with both fidelity
and SQI as hard gates. Repositioned citum-migrate output as a seed/evidence source
for embedded-core targets, not a terminal deliverable. Created new style-tune
sub-skill owning the hand-tuning loop. Updated style-qa to be tier-aware.
PR: https://github.com/citum/citum-core/pull/963
