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
| Linux (XDG) | `~/.local/share/citum/` |
| macOS | `~/Library/Application Support/Citum/` |
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

### Format transparency

The store is format-agnostic at every read/write step:

- **install** preserves the source file's format. `citum locale add foo.cbor`
  lands as `<data-dir>/locales/foo.cbor`; the file is not re-encoded. YAML is
  the one normalization: `foo.yml` lands as `foo.yaml` because the canonical
  extension for the YAML format is `yaml` (the resolver still probes both
  `.yaml` and `.yml` when looking up by id, so hand-placed `.yml` files are
  picked up too).
- **resolve** detects the format from the on-disk extension and accepts
  `yaml`, `yml`, `json`, and `cbor`. The configured `[store].format` is only
  a tiebreaker when multiple encodings of the same id coexist (e.g. both
  `apa.yaml` and `apa.cbor`).
- **list** ignores the format and reports stems.

In practice the global `[store].format` setting in `~/.config/citum/config.{yaml,toml}`
is the only knob, and most users never touch it. A per-invocation `--store-format`
flag would not change observable behavior, so it is intentionally not provided. If
you want a different on-disk encoding for an existing file, convert it first
(`citum convert locale foo.yaml -o foo.cbor`) and then install the result.

CBOR suits mobile and resource-constrained environments. JSON suits API/tooling
workflows. YAML is the default for human authoring.

### Resolution order

**Styles:** `file path → user store → http/git → registries → embedded builtins`

**Locales (file-based style):** `sibling locales/ → user store → embedded`

**Locales (builtin-alias style):** `user store → embedded`

No ambient CWD scan for styles or locales when the input is a builtin alias.

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

- [x] `citum_store` crate: `StoreResolver`, `platform_data_dir`, YAML/JSON/CBOR loading
- [x] Format config: global `~/.config/citum/config.{yaml,toml}` (per-invocation
      flag deferred — format is already transparent end-to-end; see
      "Format transparency" above)
- [x] CLI integration: `-s apa-7th` checks user store before builtins
- [x] Locale CLI integration: `-L <id>` consults the user store for both
      builtin-alias and file-based styles via the shared resolver chain
- [x] `citum store list` and `citum store install <path|slug>`
- [x] `citum store remove <name>` with confirmation

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
