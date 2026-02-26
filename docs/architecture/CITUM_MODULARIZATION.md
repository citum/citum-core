# Citum Ecosystem: Modularization and Rebranding Plan

**Status:** Approved for phased implementation
**Last updated:** 2026-02-22
**Execution runbook:** [CITUM_FULL_MIGRATION_EXECUTION_PLAN_2026-02-26.md](./CITUM_FULL_MIGRATION_EXECUTION_PLAN_2026-02-26.md)

## Overview

This document describes a phased plan to reorganize the current workspace
into a cleaner, more modular Rust ecosystem under a new GitHub organization
(`citum`) with a name that is independent of any external specification.

The rationale is operational, not relational: decoupling the project name
from an external spec gives the project independent versioning, publishing,
and API stability guarantees without requiring external coordination for
every schema or API decision.

---

## Current Architecture: Coupling Problems

The existing workspace has three boundary violations that impede clean
modularization.

### 1. `citum_schema` → `csl-legacy` (boundary violation)

`citum_schema` is the intended schema source-of-truth crate. It should have no
dependency on `csl-legacy` (a legacy XML parser). However,
`citum_schema/src/reference/conversion.rs` implements:

```rust
impl From<csl-legacy::csl_json::Reference> for InputReference { ... }
impl From<csl-legacy::csl_json::DateVariable> for EdtfString { ... }
impl From<Vec<csl-legacy::csl_json::Name>> for Contributor { ... }
```

These `From` impls belong in `citum_migrate`, not `citum_schema`. The schema
crate should define types; the migration crate should define conversions
from legacy formats into those types.

### 2. `citum_schema` → `biblatex` (belongs in processor layer)

`conversion.rs` also implements `InputReference::from_biblatex()`, which
imports `biblatex::{Chunk, Entry, Person}`. Biblatex parsing is not schema
definition. This method belongs in the processor layer (I/O or reference
conversion), where `biblatex` is already a direct dependency and where
biblatex input is actually consumed.

### 3. `citum_engine` → `clap` (unused library dependency)

`citum_engine/Cargo.toml` declares `clap` as a dependency, but no
source file in `crates/citum-engine/src/` imports it. Library crates must
not depend on CLI frameworks. This is safe to remove immediately.

---

## Target Architecture

### Dependency Graph (Clean)

```
citum-core repo:
  citum-schema    (no legacy deps: serde, schemars, csln-edtf only)
       |
  citum-engine  ---> citum-schema
       |                   |
  citum-server ------------|   [new; engine + schema only, async/http opt-in]
       |
  citum-migrate ---> citum-schema, csl-legacy   [legacy stays internal]
       |
  citum        ---> citum-engine, citum-migrate   [binary from `citum-cli` crate]
       |
  citum-bindings --> citum-engine [cdylib/wasm targets, thin wrapper only]
```

See [CITUM_SERVER_MODE.md](./CITUM_SERVER_MODE.md) for the full server mode plan.

### Crate Mapping

| Current name     | Target name      | Published? | Notes                          |
|-----------------|------------------|------------|--------------------------------|
| `citum_schema`      | `citum-schema`   | Yes        | Schema source of truth         |
| `citum_engine` | `citum-engine`   | Yes        | Rendering engine               |
| `citum_migrate`   | `citum-migrate`  | No         | Internal tooling               |
| `csl-legacy`     | `csl-legacy`     | No         | Internal tooling               |
| `csln-edtf`      | `csln-edtf`      | Yes        | Potentially standalone         |
| `citum_analyze`   | `citum-analyze`  | No         | Internal tooling               |
| `csln` (bin)     | `citum`          | Yes (bin)  | CLI binary (from `citum-cli`) |
| *(new)*          | `citum-server`   | Yes (bin)  | JSON-RPC + optional HTTP server; see [CITUM_SERVER_MODE.md](./CITUM_SERVER_MODE.md) |

### Target Workspace Layout

```
citum-core/                      # renamed from csl26
  Cargo.toml                     # workspace root
  crates/
    citum-schema/                # formerly citum_schema (minus legacy conversion)
    citum-engine/                # formerly citum_engine
    citum-migrate/               # formerly citum_migrate (absorbs conversion.rs)
    csl-legacy/                  # formerly csl-legacy (internal, not published)
    csln-edtf/                   # stays as-is
    citum-analyze/               # formerly citum_analyze
    citum-cli/                   # formerly csln (binary)
  bindings/
    lua/                         # existing LuaLaTeX integration
    latex/                       # existing LaTeX binding
    wasm/                        # future: citum-wasm (pre-production milestone)
```

---

## Implementation Phases

### Phase 0: Structural Fixes (Current, Non-Disruptive)

These changes can land now, independent of any rename. They improve the
dependency graph and correctness without breaking public APIs.

**P0-1: Remove `clap` from `citum_engine`**
- Remove `clap = { version = "4.4", ... }` from `citum_engine/Cargo.toml`
- Library crates must not depend on CLI frameworks
- Risk: none; clap is not imported in any source file

**P0-2: Decouple `citum_schema` from `csl-legacy` and `biblatex`** ✅ Done
- The `From<csl-legacy::...>` impls must stay in `citum_schema` due to the Rust
  orphan rule (`InputReference` is defined there), but `csl-legacy` is now
  optional, gated behind a `legacy-convert` feature flag
- `biblatex` is removed from `citum_schema` entirely; `from_biblatex` moved to
  `citum_engine/src/ffi.rs` as a free function (interim placement — the
  right long-term home is a dedicated IO or reference-conversion module within
  the processor, not the FFI layer)
- `citum_engine` activates the `legacy-convert` feature on `citum_schema`,
  so all existing call sites continue to work
- `citum_schema` without the feature has zero legacy deps

### Phase 1: Rename and GitHub Org (At Wave Break)

Execute at a natural pause between active style-migration waves. Renaming
mid-wave would corrupt path references in agent skills, bean tasks, and
oracle scripts.

**P1-1: Create `citum` GitHub organization**
- Transfer `csl26` → `citum/citum-core`
- Transfer `styles-hub` → `citum/citum-hub`

**P1-2: Rename crates**
- Update `package.name` fields in each `Cargo.toml`
- Rename directories to match
- Update all `path = "../..."` references in workspace `Cargo.toml`
- Add `publish = true` to `citum-schema`, `citum-engine`, `csln-edtf`
- Keep `citum-migrate`, `csl-legacy`, `citum-analyze` as `publish = false`

**P1-3: Do not publish to crates.io yet**
- Defer until schema reaches version 1.0 stability
- Use GitHub as distribution mechanism in the interim

### Phase 2: Bindings Strategy (Before Production)

**P2-1: Define `citum-bindings` public API**
- Thin wrapper over `citum-engine`
- Expose only: `render_citation`, `render_bibliography`, `validate_style`
- No internal types should leak through the public surface
- Add `wasm` feature flag with `wasm-bindgen` gated behind it

**P2-2: Create `citum/labs` repository**
- Move existing LuaLaTeX binding from `bindings/lua/` as first experiment
- Clearly document as non-stable / proof-of-concept
- Establish pattern for future experimental integrations

**P2-3 (optional, later): Evaluate `citum-server` extraction**
- Keep `citum-server` in `citum-core` during early development
- Reassess extraction only after API/protocol stabilization
- Extract only if release cadence/tooling needs diverge materially

**Do not** implement FFI tool generation (boltffi or similar) until the
engine API surface is stable. Pin to the production readiness milestone
(ROADMAP.md Phase 4) at earliest.

---

## JSON Schema Synchronization

`citum_schema` already has a `schema` feature flag using `schemars`. The JSON
Schema generated from Rust types is the mechanism for keeping `citum-hub`
and the public specification in sync. The existing `cargo run --bin citum -- schema`
command exposes this.

No new mechanism is needed. Stabilizing and publishing the schema crate
(Phase 1) is sufficient to make this path reliable.

---

## Git History on Transfer

The migration involves three git operations across two repos. The hub transfer is
independent; the labs extraction must precede the core transfer.

### Step 1: Extract citum/labs (from bindings/)

`bindings/lua/` and `bindings/latex/` move to a new `citum/labs` repository.
Subdirectory extraction requires `git filter-repo` to preserve commit history for
those paths:

```bash
# Clone a fresh copy (never filter the working repo)
git clone https://github.com/citum/citum-core.git citum-labs-extract
cd citum-labs-extract
git filter-repo --path bindings/
# Then create citum/labs on GitHub and push
gh repo create citum/labs --private
git remote set-url origin https://github.com/citum/labs.git
git push -u origin main
```

After the new repo is live, remove `bindings/` from the working tree before the
citum-core transfer. The `styles/` YAML directory stays in citum-core — it serves
as integration test fixtures and oracle reference data, not as the hub's data store
(the hub uses its own Postgres backend).

### Step 2: Transfer citum/citum-hub

`styles-hub` is an existing repo (the hub web app). This is a simple transfer, no
extraction needed:

```bash
gh repo transfer bdarcus/styles-hub citum --repo-name citum-hub
```

This can run independently of steps 1 and 3.

### Step 3: Transfer citum-core

Only after step 1 is complete (bindings/ removed from working tree), transfer the
main workspace:

```bash
gh repo transfer citum/citum-core citum --repo-name citum-core
```

GitHub preserves the full commit history and sets up automatic URL redirects
from `citum/citum-core`, so existing links do not break immediately.

### After All Steps

- Do a find-and-replace pass on hardcoded `citum/citum-core` references in `scripts/`,
  `CLAUDE.md`, and `docs/`.
- Update any `path =` references in `Cargo.toml` that pointed at the extracted
  directories.
- Do NOT use `git filter-repo` on the main working repo itself — only on throw-away
  clones. History rewriting on `citum-core` is not needed and should be avoided.

---

## Documentation Repository

Keep docs in the main `citum-core` repo for now. Docs must stay in sync with
schema changes, and a separate repo adds overhead with no benefit at this stage.

Revisit when: (a) the API reaches 1.0 stability, and (b) a dedicated
publishing pipeline (mdBook or similar) is needed for `citum-hub`. At that
point a `citum/docs` repo fed by CI from `citum-core` becomes viable.

---

## Persona Fit

| Persona         | Impact                                                     |
|----------------|-------------------------------------------------------------|
| Style Author    | None: YAML style files are unaffected by crate renaming    |
| Web Developer   | Primary beneficiary: stable `citum-schema` crate + JSON Schema |
| Systems Architect | Cleaner boundary: schema crate with no legacy deps       |
| Domain Expert   | Name independence: no version lock to external spec cycles |

---

## Related Beans

| Bean ID       | Title                                        | Phase  |
|--------------|----------------------------------------------|--------|
| `csl26-modz` | Citum modularization (epic)                  | Umbrella |
| `csl26-p0cl` | Phase 0: Remove clap from citum_engine     | 0 (now) |
| `csl26-p0dc` | Phase 0: Move csl-legacy coupling to migrate | 0      |
| `csl26-p1rn` | Phase 1: GitHub org + crate rename           | 1 (wave break) |
| `csl26-p2bn` | Phase 2: Define citum-bindings API surface   | 2      |
| `csl26-p2lb` | Phase 2: Create citum/labs repository        | 2      |
| `csl26-srvr` | Phase 2: citum-server mode (epic)            | 2      |
| `csl26-srpc` | Phase 2: Implement JSON-RPC stdin/stdout     | 2      |
| `csl26-shtp` | Phase 2: HTTP feature (axum, feature-gated)  | 2      |
