# Citum Store Architecture Plan

Bean: `csl26-erwz`
Milestone: Production Readiness (`csl26-li63`)

## Problem

Users need a place to store their own styles and locales outside the binary. The store must work across CLI, desktop, web, and mobile apps without coupling to any one platform's file I/O.

## Design

A new `citum_store` crate provides:

- `StoreResolver` — discovers styles/locales from the platform data dir, falling back to embedded builtins
- `platform_data_dir()` — returns the correct path per OS via the `dirs` crate
- Format-aware loading: styles may be stored as **YAML, JSON, or CBOR** (Citum schema only; CSL XML is not a store format)

### Platform data dirs

| Platform | Path |
|----------|------|
| Linux/macOS (XDG) | `~/.local/share/citum/` |
| Windows | `%APPDATA%\Citum\` |
| WASM / embedded | configurable override |

### Store layout

```
<data-dir>/
  styles/
    apa-7th.yaml
    my-custom.cbor
  locales/
    en-US.yaml
```

### Format configuration

The serialization format (YAML/JSON/CBOR) is configurable at three levels:

1. **Global config** — `~/.config/citum/config.toml`: `store.format = "cbor"`
2. **Per-invocation flag** — `citum render --store-format cbor ...`
3. **Default** — YAML (human-readable, easiest for hand-authoring)

CBOR is the recommended format for mobile and resource-constrained environments. JSON suits API/tooling workflows. YAML is the default for human authoring.

### Resolution order

`user store → embedded builtins`

No ambient CWD scan.

## CLI surface

```bash
citum store list                    # list user-installed styles and locales
citum store install <path>          # install from local file (name derived from stem)
citum store install <hub-slug>      # install from citum-hub (basic slug lookup)
citum store remove <name>           # remove with confirmation guard
```

Hub search/query UX (rich discovery, fidelity info, popularity) is deferred to Phase 2.

## Phasing

### Phase 1 — Core store (this epic)

- [ ] `citum_store` crate: `StoreResolver`, `platform_data_dir`, YAML/JSON/CBOR loading
- [ ] Format config: global `~/.config/citum/config.toml` + per-invocation flag
- [ ] CLI integration: `-s apa-7th` checks user store before builtins
- [ ] `citum store list` and `citum store install <path|slug>`
- [ ] `citum store remove <name>` with confirmation

### Phase 2 — Hub sync (pre-1.0)

- [ ] `citum store search <query>` — rich hub discovery
- [ ] `citum store sync` — pull updates
- [ ] Hub REST protocol types in `citum_store::hub` (feature-gated)

## Crates affected

| File | Change |
|------|--------|
| `crates/citum_store/` | CREATE new crate |
| `Cargo.toml` | Add workspace member |
| `crates/citum-cli/Cargo.toml` | Depend on `citum_store` |
| `crates/citum-cli/src/main.rs` | Wire resolver + `store` subcommand |
