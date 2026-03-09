---
# csl26-xptd
title: 'Documentation rationalization: specs/ + policies/'
status: completed
type: task
priority: normal
created_at: 2026-03-09T01:03:57Z
updated_at: 2026-03-09T01:05:35Z
---

Introduce docs/specs/ and docs/policies/ directories. Migrate TYPE_ADDITION_POLICY.md and SQI_REFINEMENT_PLAN.md. Update CLAUDE.md placement rule + Feature Design Workflow. Create doc-standards skill.

## Summary of Changes

- Created docs/policies/ and docs/specs/ with README templates and embedded doc templates
- Updated docs/README.md with links to both new directories
- Migrated docs/architecture/TYPE_ADDITION_POLICY.md → docs/policies/
- Migrated docs/architecture/SQI_REFINEMENT_PLAN.md → docs/policies/
- Updated all references in CLAUDE.md and docs/architecture/README.md
- Replaced single-line planning doc rule in CLAUDE.md with full placement table
- Added Feature Design Workflow subsection (spec-first development)
- Created ~/.claude/skills/doc-standards/SKILL.md with templates, checklist, and workflow
