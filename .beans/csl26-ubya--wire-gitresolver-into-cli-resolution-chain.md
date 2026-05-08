---
# csl26-ubya
title: Wire GitResolver into CLI resolution chain
status: todo
type: bug
priority: low
created_at: 2026-05-08T10:19:27Z
updated_at: 2026-05-08T10:19:27Z
---

GitResolver is implemented but never added to the CLI resolver chain. load_any_style() in style_resolver.rs and registry_resolvers() both omit it, so git+https:// URIs always return StyleNotFound. Fix: add GitResolver::from_platform_cache_dir() to load_any_style chain; add .with_git() when constructing RegistryResolvers. Also update prototype registry with a git+https:// entry to make the end-to-end test from the PR description actually pass. Refs: csl26-r8d2, PR #637.
