---
# csl26-m102
title: fix InputReference untagged dispatch to class-tagged
status: completed
type: bug
priority: high
created_at: 2026-02-24T17:28:07Z
updated_at: 2026-02-24T17:42:00Z
---

Replace #[serde(untagged)] on InputReference with #[serde(tag = "class")] internally-tagged dispatch. Add a 'class' string discriminant field (e.g. monograph, serial-component, collection-component, collection, legal-case, etc.) as the first field of each concrete reference struct. Update all test fixtures and existing reference YAML/JSON files. This eliminates the root cause of silent misparse bugs where serde picks the structurally-first matching variant rather than the semantically correct one. No backward-compat concerns since there are no external users.
