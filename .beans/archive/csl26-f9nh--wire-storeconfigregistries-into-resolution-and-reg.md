---
# csl26-f9nh
title: Wire StoreConfig.registries into resolution and registry update
status: completed
type: bug
created_at: 2026-05-08T09:25:25Z
updated_at: 2026-05-08T09:25:25Z
---

citum registry add and citum render both ignore registries added to config.yaml store.registries. Two fixes: (1) registry_resolvers() in style_resolver.rs should also include StoreConfig.registries entries that have cached index files; (2) run_registry_update --all should bootstrap StoreConfig.registries entries not yet in registry-sources.json.
