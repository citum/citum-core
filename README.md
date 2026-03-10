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
- `citum_schema`: schema/types and shared models
- `citum_engine`: citation and bibliography rendering engine
- `citum_migrate`: CSL 1.0 -> Citum migration pipeline (hybrid)
- `citum`: main CLI (`render`, `check`, `convert`)
- `citum_analyze`: corpus analysis tooling

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
```

## CLI Surface

`citum` currently exposes:

- `render` (subcommands: `doc`, `refs`)
- `check`
- `convert`
  - subcommands: `refs`, `style`, `citations`, `locale`

Schema generation is available with the feature-enabled build:

```bash
cargo run --bin citum --features schema -- schema style
cargo run --bin citum --features schema -- schema --out-dir ./schemas
```

## Migration Workflow (Hybrid)

Citum migration combines three approaches:

1. XML options extraction for global behavior.
2. Output-driven template inference for structure.
3. Hand-authored high-impact styles for top parent fidelity.

Run migration:

```bash
./scripts/bootstrap.sh full
./scripts/dev-env.sh cargo run --bin citum-migrate -- styles-legacy/apa.csl
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
  citum-cli/
  citum-analyze/
  citum-schema/
  csln-edtf/
  citum-migrate/
  citum-engine/

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

MPL-2.0.
