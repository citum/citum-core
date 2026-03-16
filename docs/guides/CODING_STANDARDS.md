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
