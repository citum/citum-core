---
# csl26-3he9
title: Style-level locator rendering config
status: in-progress
type: feature
created_at: 2026-03-17T18:52:06Z
updated_at: 2026-03-17T18:52:06Z
---

Replace per-template show_label/strip_label_periods fields with a style-level LocatorConfig block. Add LocatorKindConfig, LocatorPattern, LabelRepeat, and LocatorConfigEntry (preset-or-explicit) to citum-schema-style/src/options/locators.rs. Refactor engine to call a new render_locator() function in citum-engine/src/values/locator.rs. Remove show_label and strip_label_periods from TemplateVariable entirely (no backward compat). Update all affected styles, migrate fixups, and tests.
