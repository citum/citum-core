# Citum Coding Standards

## Test Style

Use the right test style based on scope and intent:

| Scenario | Location | Style |
|----------|----------|-------|
| Pure logic, type conversions, single-function correctness | `#[cfg(test)]` inline in the module | Plain `#[test]` with descriptive names |
| Single-scenario integration check | `tests/` directory | Plain `#[test]` |
| User-observable behavior with multiple parameterised cases | `tests/` directory | `#[rstest]` with `given_…_when_…_then_…` naming |

**BDD naming rule** (`given_…_when_…_then_…`) is reserved for integration tests in `tests/` that exercise a cross-module behavior path and have at least two parameterised variants. Do not apply BDD naming to inline unit tests or single-scenario integration tests — it adds noise without signal.

```rust
// Unit test — inline, plain name
#[cfg(test)]
mod tests {
    #[test]
    fn strips_trailing_period_from_title() { … }
}

// Integration test — single scenario, plain name
#[test]
fn bibliography_emits_doi_when_present() { … }

// Integration test — parameterised behavior, BDD name
#[rstest]
#[case("en-US", "Smith")]
#[case("de-DE", "Smith")]
fn given_a_multilingual_author_when_rendering_a_citation_then_family_name_is_used(
    #[case] locale: &str,
    #[case] expected: &str,
) { … }
```

## Test Independence

Tests must prove the behavior from an independent source, not mirror the current
implementation output.

- For behavior fixes, the test should fail against the old behavior unless the
  PR documents why the old behavior cannot be reproduced locally.
- Prefer expected values from literals, fixtures, oracle output, specs, or
  registered divergence decisions. Do not derive `expected` from `actual`,
  `result`, `rendered`, or other values produced by the code under test.
- Avoid weakening exact output checks to substring checks just to make a test
  pass. Use `contains` assertions only when the behavior is intentionally
  partial, order-insensitive, or format-agnostic, and make that scope clear in
  the test name or setup.
- Fixture changes must explain the missing shape they add. When fixture data
  changes expected behavior, pair it with the smallest Rust or oracle check that
  exercises the new shape.
- Ignored tests need a reason and a tracking path. Do not leave `#[ignore]`
  as an unreviewed way to keep the suite green.

Use the advisory review-smell audit before opening PRs that touch Rust tests:

```bash
python3 scripts/audit-rust-review-smells.py --changed
```

## String Ownership

Owned strings are normal at data model, serialization, FFI, and output
boundaries. They are suspicious when allocated only for lookup, comparison, or
short-lived formatting inside production paths.

- Prefer borrowed `&str` values for lookups, comparisons, and parser decisions.
  For example, avoid allocating a short `String` only to call `contains`.
- Allocate when constructing owned schema/data values, crossing FFI or serde
  boundaries, returning owned rendered output, or storing values beyond the
  borrowed input lifetime.
- Treat raw `.to_string()` counts as a triage signal only. Review categorized
  findings by path kind, especially `hot-path` production findings.
- Do not enable broad noisy Clippy restriction lints as hard failures without a
  baseline pass. Use them to inform targeted review.
- Hot-path allocation reductions are performance work: run before/after
  benchmarks and report the deltas when changing citation rendering,
  bibliography processing, style parsing, name formatting, date formatting, or
  substitution logic.

Use the advisory audit to find suspicious patterns:

```bash
python3 scripts/audit-rust-review-smells.py --all
python3 scripts/audit-rust-review-smells.py --all --json
```

## Serde Attributes Checklist

- Use `#[serde(rename_all = "kebab-case")]` for YAML/JSON compatibility
- Use `#[non_exhaustive]` for extensible enums
- Use `#[serde(deny_unknown_fields)]` on untagged enum variants to prevent misparse
- Prefer `Option<T>` with `skip_serializing_if` for optional fields
- Add `#[serde(flatten)]` for inline rendering options
- Comment non-obvious logic; reference CSL 1.0 spec where applicable

## Verification Requirements

Different types of changes require different levels of verification to maintain quality while optimizing for development velocity.

| Change Type | Verification Required |
|-------------|----------------------|
| Config/Docs/Styles (<5 lines) | Syntax check only |
| Bugfixes (non-hot path) | `cargo fmt && cargo clippy && cargo test` |
| New features (cold path) | `cargo fmt && cargo clippy && cargo test` |
| Hot path refactoring | Pre-commit checks + **Benchmarks** (before/after) |
| Algorithm changes | Pre-commit checks + **Benchmarks** (regression check) |
| Format/Parser optimization | Pre-commit checks + **Benchmarks** (validated claim) |
| Performance claims | Pre-commit checks + **Benchmarks** (evidence-based) |

**Hot paths:** citation rendering, bibliography processing, style parsing, name formatting, date formatting, substitution logic

## Benchmark Workflow (Required for Performance/Refactor Work)

Benchmarks are **required** for performance-sensitive changes and hot-path refactors. Use the provided helper script to automate comparison.

```bash
# 1. Capture baseline (on main or before changes)
./scripts/bench-check.sh capture baseline

# 2. Make performance changes
# ... implement optimization ...

# 3. Compare after changes
./scripts/bench-check.sh compare baseline after

# 4. Include relevant deltas in commit message body
```

**Available benchmarks:**
- `cargo bench --bench rendering` - Citation/bibliography processing (APA-focused)
- `cargo bench --bench formats` - YAML/JSON/CBOR deserialization

Baseline files are stored in `.bench-baselines/` (gitignored, local-only). Use `critcmp` for manual comparisons if needed.
The legacy positional forms still work for now:
`./scripts/bench-check.sh baseline` and
`./scripts/bench-check.sh baseline after`.
