---
# csl26-k2y0
title: 'Template v2 cleanup: overrides removal docs + skills'
status: completed
type: task
priority: normal
created_at: 2026-03-25T05:55:30Z
updated_at: 2026-03-25T06:00:49Z
---

Update docs/skills to reflect that TEMPLATE_V2 Step 5 (overrides removal) landed in cab0f41. Replace all overrides-as-recommendation with type-variants, add positive SQI guidance (presets first), fix spec doc, run skill-creator on affected skills.

## Summary of Changes

Updated 10 files to reflect TEMPLATE_V2 Step 5 landing in cab0f41:
- docs/specs/TEMPLATE_V2.md: v1.3→v1.4, Step 5 marked complete
- docs/architecture/DESIGN_PRINCIPLES.md: §6/§7 examples updated to type-variants; preset-first SQI guidance added
- docs/guides/style-author-guide.md + .html: overrides→type-variants throughout; preset-first SQI paragraph added
- docs/policies/SQI_REFINEMENT_PLAN.md: type-templates→type-variants; overrides guidance replaced
- .claude/contexts/{schema,processor}-context.md: overrides→type-variants in Three-Tier Options
- .claude/skills/style-{maintain,evolve}/SKILL.md: terminology updated
