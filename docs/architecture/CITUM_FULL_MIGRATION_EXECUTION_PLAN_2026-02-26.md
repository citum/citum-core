# Citum Full Migration Execution Plan (2026-02-26)

**Status:** Ready for execution at wave break  
**Scope:** End-to-end migration from `csl26`/`csln_*` naming to `citum` ecosystem naming  
**Depends on:** [CITUM_MODULARIZATION.md](./CITUM_MODULARIZATION.md), [MIGRATE_ENHANCE_PHASE3_4_SINGLE_PR_PLAN_2026-02-26.md](./MIGRATE_ENHANCE_PHASE3_4_SINGLE_PR_PLAN_2026-02-26.md)

## 1. Objective

Complete the organizational, repository, crate, symbol, binary, and documentation migration with minimal developer downtime and no fidelity regressions.

## 2. When to Execute

Run the migration only after all gates below are true:

1. **Wave gate (required):** current migration wave is complete, including acceptance criteria in [MIGRATE_ENHANCE_PHASE3_4_SINGLE_PR_PLAN_2026-02-26.md](./MIGRATE_ENHANCE_PHASE3_4_SINGLE_PR_PLAN_2026-02-26.md).
2. **Stability gate (required):** `main` has green CI and no high-priority migration fixes open for 48 hours.
3. **Freeze gate (required for multi-dev, optional solo):** if only one maintainer is active, treat this as a personal cutover window (recommended: 2-4 hours).
4. **Org/domain gate (already done):** GitHub `citum` org and `citum.org` domain are available.

## 3. Execution Strategy

Use a two-phase approach:

1. **Control-plane migration** (GitHub org/repo transfers, `labs` extraction).
2. **Data-plane migration** (mass rename of crates/symbols/commands/docs in repo).

This avoids mixing network operations and local refactors in one step.

## 4. Day-Of Runbook (Sequenced)

### Phase A: Control-plane (external)

1. Extract `bindings/` into `citum/labs` from a throwaway clone using `git filter-repo`.
2. Transfer `styles-hub` to `citum/citum-hub`.
3. Transfer `csl26` to `citum/citum-core`.
4. Confirm redirects exist for old GitHub URLs.

### Phase B: Data-plane (repo rename commit series)

1. Create a dedicated branch from new `main` (for example: `codex/citum-full-rename`).
2. Apply path renames with `git mv` (crate directories and any path-coupled folders).
3. Apply symbol/content renames using scripted replacements (see Section 5).
4. Regenerate schema/docs artifacts if they embed old binary/crate names.
5. Run verification suite (Section 7).
6. Merge after review; then remove freeze.

## 5. Efficient Mass Rename Strategy

### 5.1 Single rename map (source of truth)

Maintain one ordered mapping table and drive all replacements from it.

Suggested mapping categories:

1. **Rust crate/package names**
2. **Rust module/type references**
3. **CLI/bin names**
4. **Repository/org URLs**
5. **Docs/Bean references**

Core mappings:

| Old | New | Match mode |
|---|---|---|
| `csln_core` | `citum_schema` | word-boundary token |
| `csln_processor` | `citum_engine` | word-boundary token |
| `csln_migrate` | `citum_migrate` | word-boundary token |
| `csln_analyze` | `citum_analyze` | word-boundary token |
| `csl_legacy` | `csl_legacy` (crate path) / `csl-legacy` (package display) | targeted only |
| `crates/csln_core` | `crates/citum-schema` | path |
| `crates/csln_processor` | `crates/citum-engine` | path |
| `crates/csln_migrate` | `crates/citum-migrate` | path |
| `crates/csln_analyze` | `crates/citum-analyze` | path |
| `crates/csln` | `crates/citum-cli` | path |
| `--bin csln` | `--bin citum-cli` | command token |
| `bdarcus/csl26` | `citum/citum-core` | literal URL text |

### 5.2 Three-pass replacement model

1. **Pass 1: path rename**
   - Use `git mv` for directories/files first.
   - Update workspace members and `path =` entries.
2. **Pass 2: identifier rename**
   - Replace Rust identifiers with word-boundary-safe substitutions.
   - Rename package names in Cargo manifests.
3. **Pass 3: command/doc/url rename**
   - Replace CLI examples, README/docs links, scripts, and workflow references.

### 5.3 Safety constraints for mass rename

1. Always replace longest keys first to avoid partial collisions.
2. Use boundary-aware replacements for identifiers (avoid accidental substring edits).
3. Keep an explicit exception list for known non-migration strings:
   - CSS class names like `.csln-*` (defer to branding sweep unless intentionally changed).
   - Historical bean IDs (`csl26-*`) if task continuity is preferred.
4. Run `rg` residual scans after each pass.

## 6. Suggested Commit Slicing

Keep reviewable commits:

1. `chore(citum): transfer prep and workspace path renames`
2. `refactor(citum): crate/package/bin symbol renames`
3. `docs(citum): update commands links and architecture references`
4. `chore(citum): post-rename fixes and verification updates`

## 7. Verification Gates (Must Pass)

1. `cargo fmt`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo nextest run` (fallback: `cargo test`)
4. `cargo run --bin <new-cli-bin> -- schema` works
5. Core render command works with renamed binary and crate graph
6. `rg` residual checks:
   - no unexpected `csln_core|csln_processor|csln_migrate|csln_analyze` in active codepaths
   - no stale `bdarcus/csl26` URLs in user-facing docs/scripts (except archival docs if intentionally preserved)

## 8. Rollback and Recovery

1. Before starting Phase B, tag pre-rename state (`pre-citum-rename-YYYYMMDD`).
2. If local rename pass fails verification, hard-stop and fix forward before merge.
3. Avoid history rewrites on mainline migration branch.
4. If external transfer step fails, pause local rename and re-run transfer checks first.

## 9. Open Decisions (Resolve Before Cutover)

1. **Bean ID policy:** keep `csl26-*` IDs permanently vs introduce `citum-*` IDs for new tasks.
2. **CLI compatibility policy:** temporary shim (`csln` alias) vs immediate hard rename to `citum-cli`.
3. **Frontend branding policy:** keep `.csln-*` CSS/API classes for compatibility vs rename in same window.
4. **Server placement policy:** keep `citum-server` in `citum-core` until stabilization (default), then evaluate extraction later.

## 10. Done Criteria

Migration is complete when all are true:

1. Repositories live under `citum/*` with working redirects.
2. Workspace members, crate names, and package references are all migrated.
3. Scripts/docs/examples use new names and links.
4. Verification gates pass with no regressions.
5. Team workflows (beans, skills, oracle scripts) run without manual path/name workarounds.
