---
# csl26-tsmg
title: 'schema/engine: consolidate duplicated role-list validation and label-presentation types'
status: todo
type: task
priority: low
tags:
    - cleanup
    - contributors
created_at: 2026-07-15T10:10:08Z
updated_at: 2026-07-15T10:10:08Z
---

From PR #1052 review. (a) distinct-role validation implemented 3x with divergent error text (style/validation.rs template arm + validate_candidate_list, schema-data ContributorRoles::try_from); (b) RoleLabelPresentation duplicates RoleLabel minus term, with three near-identical structural_label_presentation unpack blocks in merged.rs::label_presentation; (c) active_roles filter duplicated between assemble_names branches; (d) sorting.rs inline EffectivePrimary::Merged branch duplicates effective_primary_names; (e) single-use SubstituteRoleLabelContext struct; (f) bib.json advertises only 'roles' — decide whether legacy 'role:' alias needs schema-level documentation.
