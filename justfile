# Citum Project justfile
# See https://github.com/casey/just for documentation

# Default recipe: run the pre-commit gate
default: pre-commit

# Run the pre-commit gate (formatting check, clippy warnings as errors, and tests)
pre-commit:
    ./scripts/dev-env.sh cargo fmt --check
    ./scripts/dev-env.sh cargo clippy --all-targets --all-features -- -D warnings
    ./scripts/dev-env.sh cargo nextest run

# Run all tests via nextest
test:
    ./scripts/dev-env.sh cargo nextest run

# Regenerate schemas when crates/citum-cli or schema crates change
schema-gen:
    ./scripts/dev-env.sh cargo run --bin citum --features schema -- schema --out-dir docs/schemas
    git add docs/schemas/

# Bootstrap the development environment (setup can be 'minimal' or 'full')
bootstrap setup="minimal":
    ./scripts/bootstrap.sh {{setup}}

# Render bibliography references using a style
render-refs style="styles/embedded/apa-7th.yaml" refs="tests/fixtures/references-expanded.json":
    ./scripts/dev-env.sh cargo run --bin citum -- render refs -s {{style}} -b {{refs}}

# Validate a style YAML and reference library file
check-style style="styles/embedded/apa-7th.yaml" refs="tests/fixtures/references-expanded.json":
    ./scripts/dev-env.sh cargo run --bin citum -- check -s {{style}} -b {{refs}}

# Validate all production styles in the repository
validate-production-styles:
    ./scripts/validate-production-styles.sh

# Convert a bibliography reference library to another format (e.g. ris, csl-json)
convert-refs input output:
    ./scripts/dev-env.sh cargo run --bin citum -- convert refs {{input}} --output {{output}}

# Run the local oracle comparison for a specific style (e.g. styles-legacy/apa.csl)
oracle style:
    node scripts/oracle.js {{style}}

# Run the oracle + batch-impact workflow test on a legacy CSL file (e.g. styles-legacy/apa.csl)
workflow-test csl:
    ./scripts/workflow-test.sh {{csl}}

# Generate a core rendering fidelity report and validate it against baseline quality gates (fails if any style's fidelity drops below 1.0)
check-core-quality:
    node scripts/report-core.js > /tmp/r.json
    node scripts/check-core-quality.js --report /tmp/r.json --baseline scripts/report-data/core-quality-baseline.json

# Refresh Top-10 oracle aggregate baselines
oracle-refresh:
    node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10

# Validate YAML frontmatter for local contributor AI skills and commands
validate-frontmatter flags='--copilot-strict':
    ./scripts/validate-frontmatter.sh {{flags}}
