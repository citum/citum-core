---
# csl26-gea2
title: Registry reverse-template index and tracing-based debug output
status: todo
type: task
priority: low
created_at: 2026-07-06T18:42:31Z
updated_at: 2026-07-06T18:42:31Z
parent: csl26-al39
---

Audit F6 (2026-07-06 migrate review): (1) discover_reverse_template_parent deserializes every embedded style per parentless migration and StyleRegistry::load_default() loads twice per resolve/promote — add a precomputed reverse-template index (or lazy static registry). (2) Debug output is split across five CITUM_MIGRATE_DEBUG* env vars plus eprintln! in synthesis modules while lineage.rs uses tracing — unify on tracing targets so RUST_LOG filtering works.
