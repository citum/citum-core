---
# csl26-93wp
title: 'Complete release pipeline: embedded assets, per-crate READMEs, JSR license'
status: in-progress
type: bug
priority: high
created_at: 2026-05-18T17:42:00Z
updated_at: 2026-05-18T18:07:19Z
blocked_by:
    - csl26-bfa9
---

## Problem

v0.53.0 release surfaced five distinct release-pipeline gaps, all of which would recur on every future tag. Need to fix once-and-for-all in a single PR before unblocking the remaining 8 crates that failed to publish.

### State after v0.53.0 partial publish

**Published to crates.io (immutable):**
- citum-edtf 0.53.0
- citum-resolver-api 0.53.0
- csl-legacy 0.53.0 *(README is the workspace README — wrong content for this crate's page)*
- citum-schema-data 0.53.0

**Blocked at packaging step (verification failed):**
- citum-schema-style, citum-schema, citum-engine, citum-io, citum_store, citum-migrate, citum-server, citum

**JSR:** publish failed — license SPDX expression `MIT OR Apache-2.0` not accepted by `deno publish`.

**GitHub Release v0.53.0:** published successfully (tarballs + SHA256SUMS + install.sh).

## Five gaps

1. **`include_bytes!` paths escape crate dir.** `crates/citum-schema-style/src/registry.rs` and `src/embedded/styles.rs` use `include_bytes!("../../../../styles/embedded/<name>.yaml")` and `include_bytes!("../../../registry/default.yaml")`. `cargo publish` packages only the crate dir, so verification fails on the tarball.

2. **`readme = "../../README.md"` escapes crate dir.** Three crates point at the workspace README: csl-legacy, citum_store, citum-server. The workspace README describes the entire Citum project — wrong content for an individual crate's crates.io page. Even apart from content, the path-escape pattern has the same packaging issue as gap 1.

3. **Missing per-crate READMEs.** 9 of 12 publishable crates have no README.md at all. Their crates.io pages will show only the Cargo.toml description.

4. **citum umbrella crate has no `description`.**

5. **JSR license syntax.** `(MIT OR Apache-2.0)` (parens) is the next thing to try; SPDX expressions without parens are rejected.

## Plan

### 1. Embedded asset relocation (gap 1)

Move physical source-of-truth into the crate, symlink the old workspace paths back so the ~15 external readers see no change.

- Files relocate:
  - `styles/embedded/*.yaml` → `crates/citum-schema-style/embedded/styles/*.yaml`
  - `registry/default.yaml` → `crates/citum-schema-style/embedded/registry/default.yaml`
- Workspace paths become symlinks pointing into the crate:
  - `styles/embedded` → `crates/citum-schema-style/embedded/styles`
  - `registry/default.yaml` → `crates/citum-schema-style/embedded/registry/default.yaml`
- Source path updates inside the crate:
  - `src/registry.rs`: `include_bytes!("../embedded/registry/default.yaml")`
  - `src/embedded/styles.rs`: `include_bytes!("../embedded/styles/<name>.yaml")`
- No mirror script, no drift check needed — symlinks ARE the link.

Windows is not in play: CI runs only on ubuntu-latest, and the Windows release-binary build (`cargo build --bin citum`) reads `include_bytes!` paths that are wholly inside the crate, never traversing the workspace-level symlinks.

### 2. Per-crate READMEs (gaps 2 + 3)

Write a focused README.md for each of the 12 publishable crates. Each contains:

- One-paragraph purpose (more specific than the Cargo.toml description).
- Concrete usage snippet where applicable — libs get a Rust snippet; binaries get a command-line example.
- Link back to the workspace README for full project context.
- License footer (dual MIT / Apache-2.0).

Wire `readme = "README.md"` in every `Cargo.toml`. Remove every `readme = "../../README.md"` indirection.

### 3. Add citum description (gap 4)

Set `description = "Citum citation engine — CLI + reference distribution"` (or equivalent) in `crates/citum/Cargo.toml`.

### 4. JSR license fix (gap 5)

`scripts/build-jsr-package.sh`: `"license": "(MIT OR Apache-2.0)"` (already staged on this branch).

### 5. CI dry-run check

New job in `.github/workflows/ci.yml` that runs `scripts/publish-crates.sh --dry-run` when `crates/**/Cargo.toml`, `crates/**/src/**`, or `scripts/publish-crates.sh` change on a PR. Catches all five gap categories pre-merge, prevents future surprise tag-time failures.

### 6. Ship as v0.53.1

After PR merges:
- Tag-driven release fires.
- Skips already-published citum-edtf/resolver-api/csl-legacy/schema-data at 0.53.0 (idempotent script — but at 0.53.1 they re-publish *with* READMEs).
- All 8 previously-blocked crates publish successfully at 0.53.1.
- JSR publishes 0.53.1.

## Tasks

- [x] Implement gap 1: moved styles+locales+registry into crate, symlinks at workspace, include_bytes paths updated
- [x] Implement gap 2 + 3: 9 new READMEs + 3 existing kept; all 12 Cargo.toml wired to readme = README.md
- [x] Verified gap 4: citum crate already has description (was at crates/citum-cli/Cargo.toml; original audit was looking at wrong path)
- [x] Implement gap 5: jsr.json license parens (MIT OR Apache-2.0); local JSR dry-run passes
- [x] Implement CI: release-dry-runs job already existed; expanded release paths filter from narrow allowlist to all crates/** so the gate fires on any publishable-crate change
- [x] Local validation: `cargo publish --dry-run` on every crate from a clean tree
- [ ] PR with Copilot review
- [ ] After merge: cut v0.53.1 via the release PR workflow

## Out of scope

- Yanking the four already-published v0.53.0 crates. crates.io is immutable; they get proper READMEs at v0.53.1.
- Restructuring `styles/embedded/` to live inside the crate (rejected — 14+ external consumers; high churn for marginal gain).
