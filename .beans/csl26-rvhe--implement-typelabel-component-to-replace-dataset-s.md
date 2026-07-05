---
# csl26-rvhe
title: Implement TypeLabel component to replace [Dataset] suffix hack
status: todo
type: task
created_at: 2026-07-05T13:23:57Z
updated_at: 2026-07-05T13:23:57Z
parent: csl26-8m2p
blocked_by:
    - csl26-92mg
---

Part B of docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md: add TemplateComponent::TypeLabel (localized reference-type term with genre/medium fallback), rewrite apa-7th.yaml's dataset variant to drop the suffix: " [Dataset]." literal and use type-label + wrap: brackets instead, remove the engine de-dup at render/component.rs, regenerate schemas (STYLE_SCHEMA_VERSION minor bump via feat commit), update fixtures in tests/domain_fixtures.rs and tests/bibliography.rs. Confirm the exact label term text with the user before finalizing.
