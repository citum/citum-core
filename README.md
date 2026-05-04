# Citum

Citum is a Rust-based, declarative citation styling system.

It is the successor-focused evolution of CSL 1.0: styles are expressed as YAML templates and options, then rendered by a type-safe processor with oracle verification against `citeproc-js`.

## Status

Citum is in active development.

For current, generated metrics, use these as source of truth:

- Compatibility dashboard: [`citum.github.io/citum-core/compat.html`](https://citum.github.io/citum-core/compat.html)
- Tier status snapshot: [`citum.github.io/citum-core/TIER_STATUS.md`](https://citum.github.io/citum-core/TIER_STATUS.md)
- Core fidelity/SQI baseline: [`scripts/report-data/core-quality-baseline.json`](./scripts/report-data/core-quality-baseline.json)

Do not treat hard-coded README percentages as canonical.

## What Citum Includes

- `csl-legacy`: CSL 1.0 XML parser
- `citum_schema`: schema/types and shared models (includes `citum-schema-data`, `citum-schema-style`)
- `citum_engine`: citation and bibliography rendering engine
- `citum_migrate`: CSL 1.0 -> Citum migration pipeline (hybrid)
- `citum-cli`: main binary (`citum`) and subcommands
- `citum_analyze`: corpus analysis tooling
- `citum-bindings`: FFI and language bindings (JS/WASM)
- `citum-edtf`: native EDTF date processing
- `citum-pdf`: PDF generation via Typst
- `citum-server`: stateless JSON-RPC server
- `citum_store`: user-level persistent registry and store

## Quick Start

```bash
git clone https://github.com/citum/citum-core
cd citum-core
./scripts/bootstrap.sh minimal
./scripts/dev-env.sh cargo build --workspace
./scripts/dev-env.sh cargo test --workspace
```

The default local setup is intentionally lean:

- `./scripts/bootstrap.sh minimal` installs script dependencies without fetching the legacy CSL corpora.
- `./scripts/dev-env.sh <command>` keeps `CARGO_TARGET_DIR` outside the repo at `${XDG_CACHE_HOME:-$HOME/.cache}/citum-core/target`.
- Run `./scripts/bootstrap.sh full` only when you need migration, oracle, or compatibility-report workflows that depend on `styles-legacy/` or `tests/csl-test-suite/`.

Render references:

```bash
cargo run --bin citum -- render refs \
  -b tests/fixtures/references-expanded.json \
  -s styles/apa-7th.yaml
```

Render a document:

```bash
cargo run --bin citum -- render doc \
  -i examples/document.djot \
  -b examples/document-refs.json \
  -s styles/apa-7th.yaml \
  -I djot -O html
```

Validate inputs:

```bash
cargo run --bin citum -- check \
  -s styles/apa-7th.yaml \
  -b tests/fixtures/references-expanded.json \
  -c tests/fixtures/citations-expanded.json
```

Validate all production styles with the workspace binary:

```bash
./scripts/validate-production-styles.sh
```

Convert formats:

```bash
cargo run --bin citum -- convert style styles/apa-7th.yaml --output /tmp/apa-7th.cbor
cargo run --bin citum -- convert refs tests/fixtures/references-expanded.json --output /tmp/refs.ris
cargo run --bin citum -- convert refs /tmp/refs.ris --output /tmp/refs.json --to citum-json
```

`convert refs` supports `citum-yaml`, `citum-json`, `citum-cbor`, `csl-json`, `biblatex`, and `ris`.
RIS multiline field continuations are preserved during parsing, and CSL `issued` dates are emitted as year
when parseable or `literal` otherwise to avoid dropping date semantics.

## CLI Surface

`citum` currently exposes:

- `render` (subcommands: `doc`, `refs`)
- `check`
- `convert` (subcommands: `refs`, `style`, `citations`, `locale`)
- `styles` (subcommands: `list`)
- `registry` (subcommands: `list`, `resolve`)
- `store` (subcommands: `list`, `install`, `remove`)
- `style` (subcommand: `lint`)
- `locale` (subcommand: `lint`)

Schema generation is available with the feature-enabled build:

```bash
cargo run --bin citum --features schema -- schema style
cargo run --bin citum --features schema -- schema --out-dir ./schemas
```

## Migration Workflow (Hybrid)

Citum migration combines output-driven template inference with XML extraction:

1. Extract global options from CSL XML.
2. Resolve citation and bibliography templates from inferred output artifacts (cache first, then live inference).
3. Fall back to XML template compilation only when template artifacts are missing or rejected.
4. Emit `extends:`-based wrapper styles when the target lineage matches a known profile or journal.

Run a single-style migration (uses embedded Deno-based JS runtime for live inference):

```bash
./scripts/bootstrap.sh full
./scripts/dev-env.sh cargo run --bin citum-migrate -- styles-legacy/apa.csl
```

Prepare a batch inference cache (requires Node.js; allows subsequent Rust migrations without live JS):

```bash
./scripts/bootstrap.sh full
./scripts/batch-infer.sh
```

Migrate using the cache-only inferred mode:

```bash
cargo run --bin citum-migrate -- styles-legacy/apa.csl --template-source inferred
```

Prepare high-fidelity authoring context:

```bash
./scripts/bootstrap.sh full
./scripts/prep-migration.sh styles-legacy/apa.csl
```

Detailed migration docs:

- [`crates/citum-migrate/README.md`](./crates/citum-migrate/README.md)
- [`docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md`](./docs/architecture/MIGRATION_STRATEGY_ANALYSIS.md)

## Verification Workflow

Single-style oracle checks:

```bash
./scripts/bootstrap.sh full
node scripts/oracle.js styles-legacy/apa.csl
node scripts/oracle-e2e.js styles-legacy/apa.csl
```

Top-style aggregate:

```bash
./scripts/bootstrap.sh full
node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10
```

Core fidelity + SQI gate:

```bash
./scripts/bootstrap.sh full
node scripts/report-core.js > /tmp/core-report.json
node scripts/check-core-quality.js \
  --report /tmp/core-report.json \
  --baseline scripts/report-data/core-quality-baseline.json
```

Production style validity gate:

```bash
./scripts/validate-production-styles.sh
```

During development, use `cargo run --bin citum -- ...` or
`./scripts/validate-production-styles.sh` as the authoritative validation path.
A globally installed `citum` binary may lag the current workspace build and can
report stale style failures until it is rebuilt or reinstalled.

## Repository Layout

```text
crates/
  csl-legacy/
  citum-schema-data/
  citum-schema-style/
  citum-schema/
  citum-migrate/
  citum-engine/
  citum-analyze/
  citum-cli/
  citum-pdf/
  citum-server/
  citum-edtf/
  citum_store/
  citum-bindings/

docs/
styles/
styles-legacy/      # Optional submodule; fetch with ./scripts/bootstrap.sh full
scripts/
tests/
tests/csl-test-suite/  # Optional submodule; fetch with ./scripts/bootstrap.sh full
```

## Documentation Map

- Rendering workflow: [`docs/guides/RENDERING_WORKFLOW.md`](./docs/guides/RENDERING_WORKFLOW.md)
- Style tier tracking: [`docs/TIER_STATUS.md`](./docs/TIER_STATUS.md)
- Design and architecture docs: [`docs/architecture/`](./docs/architecture/)
- Web docs entry point: [`docs/index.html`](./docs/index.html)

## Contributing

- For roadmap/design context, start in [`docs/architecture/`](./docs/architecture/).
- For rendering issues, follow [`docs/guides/RENDERING_WORKFLOW.md`](./docs/guides/RENDERING_WORKFLOW.md).
- For local task tracking, see `.beans/` and project workflow docs.
- Use `./scripts/bootstrap.sh minimal` for default setup and `./scripts/bootstrap.sh full` only for corpus-backed workflows.
- Use `./scripts/dev-env.sh <command>` for local cargo commands to keep build artifacts out of the repo.

If your change touches Rust code (`.rs`, `Cargo.toml`, `Cargo.lock`), run:

```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run
```

If `cargo nextest` is unavailable, use:

```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

## License

MIT or Apache 2.0
