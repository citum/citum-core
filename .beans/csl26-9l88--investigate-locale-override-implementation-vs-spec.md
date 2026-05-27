---
# csl26-9l88
title: investigate locale override implementation vs spec
status: todo
type: task
created_at: 2026-05-27T17:45:47Z
updated_at: 2026-05-27T17:45:47Z
---

Spec says options.locale 'replaces the style default locale entirely'. Copilot noted config.locale-override is currently a patch ID, not a base-locale selector. Investigate whether de-DE resolves correctly at runtime via the CLI --locale path, or whether a separate document-level base-locale mechanism is needed.
