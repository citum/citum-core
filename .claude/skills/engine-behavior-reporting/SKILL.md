---
name: engine-behavior-reporting
description: >
  Expand or maintain the engine behavior coverage report in citum-core. Use
  this skill when converting additional citum-engine integration suites into
  behavior-oriented tests, adding them to the nextest-backed Markdown/HTML
  report, refining behavior summaries, or updating the published docs report
  without increasing PR CI time.
---

# Engine Behavior Reporting

**Type:** User-Invocable, Agent-Invocable
**LLM Access:** Yes
**Purpose:** Keep engine behavior tests readable to humans and keep the generated behavior report aligned with those tests.

## Use This Skill When
- You are converting another `crates/citum-engine/tests/*.rs` integration suite into behavior-oriented scenarios.
- You are adding another engine suite to the generated behavior report.
- You are improving `announce_behavior(...)` summaries for report quality.
- You are updating the docs publication path for the behavior coverage page.

## Do Not Use This Skill When
- You are changing low-level unit tests under `src/`.
- You are trying to make `cargo nextest run` itself into the human report.
- You are expanding coverage outside `citum-engine` integration suites.

## Current Report Scope
The published report currently covers:

- `crates/citum-engine/tests/citations.rs`
- `crates/citum-engine/tests/document.rs`
- `crates/citum-engine/tests/i18n.rs`

## Expansion Workflow
1. Pick the next engine integration suite with clear user-visible behavior.
   Prefer acceptance-style suites over low-level internals.
2. Rewrite that suite to match the pilot shape:
   - group related scenarios into modules
   - keep scenario names readable
   - call `announce_behavior(...)` with one plain sentence per scenario
   - keep setup in shared helpers where possible
3. Ensure the scenario text describes observable behavior, not implementation trivia.
   Good: "A grouped bibliography should restart year suffixes inside each group."
   Bad: "Calls processor.process_document and asserts the result matches snapshot."
4. Add the suite to the reporting pipeline:
   - add its binary-to-source mapping in `scripts/generate-test-report.py`
   - add it to the default target list in `scripts/test-report.sh` only if it should be part of the default published engine report
5. Regenerate the report with `./scripts/test-report.sh`.
6. Read `target/test-report.md` or `target/test-report.html` and confirm it is useful to a human reviewer without opening source.

## Source Reference Rules
- The report should emit concise repo-relative source references like `crates/citum-engine/tests/foo.rs:120-128`.
- Do not publish generated source-browser pages.
- Do not add GitHub-specific links.

## CI Rule
- Do not lengthen normal PR CI for this feature.
- Keep behavior-report publication in `main`-only or manual docs workflows unless the user explicitly asks for PR-time publication.

## Files To Update
- `crates/citum-engine/tests/common/mod.rs`
- `crates/citum-engine/tests/*.rs`
- `scripts/generate-test-report.py`
- `scripts/test-report.sh`
- `docs/index.html`
- `docs/README.md`
- `.github/workflows/compat-report.yml`
- `.github/workflows/deploy_docs.yml`

## Validation
For report-only or docs-only changes:

- `./scripts/validate-frontmatter.sh --repo-only --copilot-strict`
- `./scripts/test-report.sh`

For Rust-touching changes in test files or Cargo manifests:

- `cargo fmt`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run`
- `./scripts/test-report.sh`

## Definition Of Done
- The converted suite is readable on its own.
- The generated report shows behavior bullets, not machine-oriented inventory.
- Source references point to the right file and line range.
- The docs publication path still avoids extending normal PR CI time.
