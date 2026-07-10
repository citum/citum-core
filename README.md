# Citum

Citum is a citation engine with a richer reference data model and style language than CSL 1.0 can express. It is delivered as a portable Rust library that runs on any surface — CLI, WASM, JSON-RPC server, or C FFI. Styles are declarative YAML, validated at load time, with rendering oracle-verified against the established CSL and biblatex ecosystems. See [capabilities](https://docs.citum.org/examples.html) for a full feature overview.

> **Researchers and style authors:** see [citum.org](https://citum.org) and [docs.citum.org](https://docs.citum.org) instead of this document.

## Install

Prebuilt binaries (Linux, macOS arm64, Windows via Git Bash):

```bash
curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | sh
```

The installer verifies SHA-256 checksums and installs to `~/.local/bin`. Select components with `CITUM_COMPONENTS` (`citum`, `citum-server`, `citum-migrate`, or `all`):

```bash
curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | CITUM_COMPONENTS=all sh
```

From source: `cargo install citum --locked` (also: `citum-server`, `citum-migrate`).

JavaScript/TypeScript (WASM): `npx jsr add @citum/engine`

## Why Citum

- **Richer reference and style model** — expressive reference types, dates, and relationships beyond what CSL 1.0 can represent; see [capabilities](https://docs.citum.org/examples.html)
- **Declarative YAML styles** — human-readable, diff-friendly, and toolable; no procedural XML
- **Type-safe schema** — styles are fully validated at load time; invalid styles are rejected, not silently misrendered
- **Oracle-verified rendering** — parity targets against citeproc-js (CSL ecosystem) and biblatex; regressions caught automatically
- **Deploy anywhere** — the same engine runs as a CLI binary, a WASM module, a JSON-RPC server, or via C FFI

## Pipeline

```
References (JSON / YAML / BibLaTeX / RIS)
         │
         ▼
   citum-schema  ←── Style (YAML)  ←── Locale (YAML)
         │
         ▼
   citum-engine
         │
         ▼
   HTML / plain text / LaTeX / Typst / Djot / …

   CSL 1.0 XML ──► citum-migrate ──► Style (YAML)   (migration path)
```

## Status

Citum is in active development. The schema, engine, and CLI are the stable core; PDF output and some language bindings are experimental.

For live metrics — do not rely on any hardcoded numbers in this file:

- Compatibility dashboard: [`citum.github.io/citum-core/compat.html`](https://citum.github.io/citum-core/compat.html)

## For App Developers

Citum exposes multiple integration surfaces from the same engine:

| Surface | Crate | Notes |
|---|---|---|
| WASM | `citum-bindings` | Node-compatible; see [citum-hub](https://github.com/citum/citum-hub) for the reference WASM integration |
| JSON-RPC server | `citum-server` | Stateless HTTP/JSON-RPC; suitable for sidecar or hosted deployment |
| C FFI | `citum-bindings` | Used by [citum-labs](https://github.com/citum/citum-labs) for Lua and Python bindings |
| Rust library | `citum-engine` / `citum-schema` | Direct crate dependency |

Rust API docs: [docs.rs/citum-engine](https://docs.rs/citum-engine). JS/TS bindings: [jsr.io/@citum](https://jsr.io/@citum).

## For Contributors

### Quick Start

```bash
git clone https://github.com/citum/citum-core
cd citum-core
./scripts/bootstrap.sh minimal          # lean setup, no legacy corpora
./scripts/dev-env.sh cargo build --workspace
./scripts/dev-env.sh cargo test --workspace
```

For orchestrating common developer tasks (testing, quality gates, schema generation, etc.), this repository supports [just](https://github.com/casey/just). Run `just` at the root to see available recipes.

`./scripts/bootstrap.sh full` fetches the `styles-legacy/` and tests/csl-test-suite submodules needed for migration and oracle workflows.

`./scripts/dev-env.sh <cmd>` keeps `CARGO_TARGET_DIR` outside the repo at `${XDG_CACHE_HOME:-$HOME/.cache}/citum-core/target`.

### Key Commands

Render references:

```bash
just render-refs styles/apa-7th.yaml tests/fixtures/references-expanded.json
```

Alternatively, run the raw command:

```bash
cargo run --bin citum -- render refs \
  -b tests/fixtures/references-expanded.json \
  -s styles/apa-7th.yaml
```

Validate a style and references:

```bash
just check-style styles/apa-7th.yaml tests/fixtures/references-expanded.json
```

Alternatively, run the raw command:

```bash
cargo run --bin citum -- check \
  -s styles/apa-7th.yaml \
  -b tests/fixtures/references-expanded.json
```

Validate all production styles:

```bash
just validate-production-styles
```

Alternatively, run the script directly:

```bash
./scripts/validate-production-styles.sh
```

Convert reference formats:

```bash
just convert-refs tests/fixtures/references-expanded.json /tmp/refs.ris
```

Alternatively, run the raw command:

```bash
cargo run --bin citum -- convert refs tests/fixtures/references-expanded.json \
  --output /tmp/refs.ris
```

`convert refs` supports `citum-yaml`, `citum-json`, `citum-cbor`, `csl-json`, `biblatex`, and `ris`.
Run `citum --help` for the full command surface.

### Crate Map

See [crates/README.md](./crates/README.md) for the workspace layout and where work usually happens.

### Pre-Commit Gate

Before committing `.rs`, `Cargo.toml`, or `Cargo.lock`, run:

```bash
just pre-commit
```

Alternatively, run the raw commands:

```bash
./scripts/dev-env.sh cargo fmt --check
./scripts/dev-env.sh cargo clippy --all-targets --all-features -- -D warnings
./scripts/dev-env.sh cargo nextest run   # fallback: cargo test
```

Run `cargo fmt` (without `--check`) first if formatting is dirty, then re-check. Do not commit if any check fails.

### Commit Conventions

Conventional Commits (`type(scope): subject`, lowercase, 50/72 rule). See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for allowed scopes, versioning signals, and PR workflow.

## Documentation

| Resource | Location |
|---|---|
| Full user docs | [docs.citum.org](https://docs.citum.org) |
| Specifications | [docs/specs/](./docs/specs/) |
| Rendering workflow | [docs/guides/RENDERING_WORKFLOW.md](./docs/guides/RENDERING_WORKFLOW.md) |
| Style authoring guide | [docs/guides/style-author-guide.md](./docs/guides/style-author-guide.md) |
| Migration docs | [crates/citum-migrate/README.md](./crates/citum-migrate/README.md) |
| Contributing | [CONTRIBUTING.md](./CONTRIBUTING.md) |

## License

MIT or Apache 2.0
