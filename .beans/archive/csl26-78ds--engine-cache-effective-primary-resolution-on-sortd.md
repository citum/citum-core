---
# csl26-78ds
title: 'engine: cache effective-primary resolution on sort/disambiguation paths'
status: completed
type: task
priority: normal
tags:
    - engine
    - performance
created_at: 2026-07-15T10:10:08Z
updated_at: 2026-07-15T15:19:42Z
---

From PR #1052 review. extract_author_sort_key_opt re-runs effective_primary/semantic_names (+ fresh Config::default()) per pairwise comparison inside sort_by comparators (disambiguation.rs:777, setup.rs:625); effective_primary_names computes semantic_names twice (emptiness probe discards result); per-entry clones: TemplateContributor clone per run in resolve_entry_label, ContributorMerge HashMap clone per effective_merge call, roles.to_vec() per name in append_contributor. CachedReference/cache_sort_value machinery exists for exactly this. Benchmarks currently fine — latent cost on large bibliographies with list primaries.

## Summary of Changes

Single-pass effective-primary resolution (EffectivePrimaryResolution carries names; effective_primary/effective_primary_names are thin wrappers — no double semantic_names). Generic ReferenceSorter::sort_by_keys precomputes per-item sort keys (Schwartzian transform, shared compare_cached_values) replacing per-comparison compare_by_key at disambiguation year-suffix sort and setup citation-item sort. ContributorConfig::effective_merge and merged.rs effective_merge return Cow, cloning only for the default. Skipped by design: Rc role slices, per-entry component clone shaving. Criterion vs pre-change baseline: citation processing −14%, bibliography benches −27% to −47%. Folded into PR #1052 feat commit.
