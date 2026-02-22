# Citum Ecosystem: Modularization and Rebranding Plan

**Status:** Approved for phased implementation
**Last updated:** 2026-02-22

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

### 1. `csln_core` → `csl_legacy` (boundary violation)

`csln_core` is the intended schema source-of-truth crate. It should have no
dependency on `csl_legacy` (a legacy XML parser). However,
`csln_core/src/reference/conversion.rs` implements:

```rust
impl From<csl_legacy::csl_json::Reference> for InputReference { ... }
impl From<csl_legacy::csl_json::DateVariable> for EdtfString { ... }
impl From<Vec<csl_legacy::csl_json::Name>> for Contributor { ... }
```

These `From` impls belong in `csln_migrate`, not `csln_core`. The schema
crate should define types; the migration crate should define conversions
from legacy formats into those types.

### 2. `csln_core` → `biblatex` (belongs in migration layer)

`conversion.rs` also implements `InputReference::from_biblatex()`, which
imports `biblatex::{Chunk, Entry, Person}`. Biblatex parsing is migration
tooling, not schema definition. This method should move to `csln_migrate`.

### 3. `csln_processor` → `clap` (unused library dependency)

`csln_processor/Cargo.toml` declares `clap` as a dependency, but no
source file in `crates/csln_processor/src/` imports it. Library crates must
not depend on CLI frameworks. This is safe to remove immediately.

---

## Target Architecture

### Dependency Graph (Clean)

```
citum-schema    (no legacy deps: serde, schemars, csln_edtf only)
     |
citum-engine  ---> citum-schema
     |
citum-migrate ---> citum-schema, csl-legacy   [legacy stays internal]
     |
citum-cli     ---> citum-engine, citum-migrate
     |
citum-bindings --> citum-engine [cdylib/wasm targets, thin wrapper only]
```

### Crate Mapping

| Current name     | Target name      | Published? | Notes                          |
|-----------------|------------------|------------|--------------------------------|
| `csln_core`      | `citum-schema`   | Yes        | Schema source of truth         |
| `csln_processor` | `citum-engine`   | Yes        | Rendering engine               |
| `csln_migrate`   | `citum-migrate`  | No         | Internal tooling               |
| `csl_legacy`     | `csl-legacy`     | No         | Internal tooling               |
| `csln_edtf`      | `csln-edtf`      | Yes        | Potentially standalone         |
| `csln_analyze`   | `citum-analyze`  | No         | Internal tooling               |
| `csln` (bin)     | `citum-cli`      | Yes (bin)  | CLI binary                     |

### Target Workspace Layout

```
citum-core/                      # renamed from csl26
  Cargo.toml                     # workspace root
  crates/
    citum-schema/                # formerly csln_core (minus legacy conversion)
    citum-engine/                # formerly csln_processor
    citum-migrate/               # formerly csln_migrate (absorbs conversion.rs)
    csl-legacy/                  # formerly csl_legacy (internal, not published)
    csln-edtf/                   # stays as-is
    citum-analyze/               # formerly csln_analyze
    citum-cli/                   # formerly csln (binary)
  bindings/
    lua/                         # existing LuaLaTeX integration
    latex/                       # existing LaTeX binding
    wasm/                        # future: citum-wasm (Phase 4+)
```

---

## Implementation Phases

### Phase 0: Structural Fixes (Current, Non-Disruptive)

These changes can land now, independent of any rename. They improve the
dependency graph and correctness without breaking public APIs.

**P0-1: Remove `clap` from `csln_processor`**
- Remove `clap = { version = "4.4", ... }` from `csln_processor/Cargo.toml`
- Library crates must not depend on CLI frameworks
- Risk: none; clap is not imported in any source file

**P0-2: Move `csl_legacy` conversion impls to `csln_migrate`**
- Move `From<csl_legacy::...>` impls from `csln_core/src/reference/conversion.rs`
  to `csln_migrate`
- Move `InputReference::from_biblatex()` to `csln_migrate` as a free function
- Remove `csl_legacy` and `biblatex` deps from `csln_core/Cargo.toml`
- This is the primary architectural fix; `csln_core` becomes a clean schema crate
- Risk: medium; requires updating all call sites for `from_biblatex` and the `From` impls

### Phase 1: Rename and GitHub Org (At Wave Break)

Execute at a natural pause between active style-migration waves. Renaming
mid-wave would corrupt path references in agent skills, bean tasks, and
oracle scripts.

**P1-1: Create `citum` GitHub organization**
- Transfer `csl26` → `citum/citum-core`
- Transfer `style-hub` → `citum/citum-hub`

**P1-2: Rename crates**
- Update `package.name` fields in each `Cargo.toml`
- Rename directories to match
- Update all `path = "../..."` references in workspace `Cargo.toml`
- Add `publish = true` to `citum-schema`, `citum-engine`, `csln-edtf`
- Keep `citum-migrate`, `csl-legacy`, `citum-analyze` as `publish = false`

**P1-3: Do not publish to crates.io yet**
- Defer until schema reaches version 1.0 stability
- Use GitHub as distribution mechanism in the interim

### Phase 2: Bindings Strategy (Before Phase 4)

**P2-1: Define `citum-bindings` public API**
- Thin wrapper over `citum-engine`
- Expose only: `render_citation`, `render_bibliography`, `validate_style`
- No internal types should leak through the public surface
- Add `wasm` feature flag with `wasm-bindgen` gated behind it

**P2-2: Create `citum/labs` repository**
- Move existing LuaLaTeX binding from `bindings/lua/` as first experiment
- Clearly document as non-stable / proof-of-concept
- Establish pattern for future experimental integrations

**Do not** implement FFI tool generation (boltffi or similar) until the
engine API surface is stable. Pin to Phase 4 at earliest.

---

## JSON Schema Synchronization

`csln_core` already has a `schema` feature flag using `schemars`. The JSON
Schema generated from Rust types is the mechanism for keeping `citum-hub`
and the public specification in sync. The existing `cargo run --bin csln -- schema`
command exposes this.

No new mechanism is needed. Stabilizing and publishing the schema crate
(Phase 1) is sufficient to make this path reliable.

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
| `csl26-p0cl` | Phase 0: Remove clap from csln_processor     | 0 (now) |
| `csl26-p0dc` | Phase 0: Move csl_legacy coupling to migrate | 0      |
| `csl26-p1rn` | Phase 1: GitHub org + crate rename           | 1 (wave break) |
| `csl26-p2bn` | Phase 2: Define citum-bindings API surface   | 2      |
| `csl26-p2lb` | Phase 2: Create citum/labs repository        | 2      |
