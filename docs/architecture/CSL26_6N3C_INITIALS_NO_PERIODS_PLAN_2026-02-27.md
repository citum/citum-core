# CSL26-6N3C Fix Plan: Name Initials Without Periods

Date: 2026-02-27  
Bean: `csl26-6n3c`  
Scope: `crates/citum-engine` (name rendering), targeted tests

## Problem
Some styles require initials without periods.

Examples:
- With period-space: `Kuhn, T. S.`
- With no period: `Kuhn TS`
- With space only separator between initials: `Kuhn T S`

Bean requirements:
- Handle `initialize-with = ""` (compact initials, no punctuation)
- Handle `initialize-with = " "` (space-separated initials)
- Distinguish from `initialize-with = "."` and `". "`

## Current State
The initials logic is implemented in:
- `crates/citum-engine/src/values/contributor.rs` (`format_single_name`)

Existing coverage includes disambiguation scenarios with `initialize_with = " "`, but current tests do not explicitly cover multi-token given names where whitespace separators can be duplicated.

## Root Cause Hypothesis
In `format_single_name`, when `initialize_with` contains whitespace (e.g., `" "`), the algorithm appends:
1. the initialized token plus `initialize_with`, and
2. the original separator when that separator is whitespace.

For multi-part given names (e.g., `"Thomas Samuel"`), this can produce doubled spaces (`"T  S"`) instead of normalized single-space output (`"T S"`).

## Implementation Plan
1. Add failing unit tests first in `crates/citum-engine/src/values/tests.rs`.
   - Case A: `initialize_with = ""` + multi-part given name -> compact initials (`"TS"`)
   - Case B: `initialize_with = " "` + multi-part given name -> single-space initials (`"T S"`)
   - Case C: `initialize_with = "."` remains period-only (`"T.S."`)
   - Case D: `initialize_with = ". "` remains period-space (`"T. S."`)
   - Case E (hyphen behavior guard): verify `initialize_with_hyphen = Some(false)` behavior does not regress.

2. Refactor initials tokenization in `format_single_name`.
   - Parse `given` into logical name parts and separators once.
   - Build initials from parts and apply separator policy explicitly instead of conditionally echoing raw separators.
   - Separator policy:
     - For `initialize_with = ""`: no extra separator unless explicit hyphen retention is required by CSL hyphen rules.
     - For `initialize_with = " "`: normalize to single spaces between initialized parts.
     - For period variants (`"."`, `". "`): preserve current rendered behavior.

3. Keep behavior strictly scoped.
   - No schema changes.
   - No migration-pipeline changes.
   - No style preset changes unless a failing regression proves mismatch.

4. Validate across targeted and broader tests.
   - Run targeted tests for contributor formatting and citation disambiguation.
   - Run full Rust gate before commit:
     - `cargo fmt`
     - `cargo clippy --all-targets --all-features -- -D warnings`
     - `cargo nextest run` (fallback `cargo test` if nextest unavailable)

## Acceptance Criteria
- `initialize_with = ""` renders compact initials with no periods and no injected spaces.
- `initialize_with = " "` renders exactly one space between initials.
- `initialize_with = "."` and `". "` outputs remain unchanged from current behavior.
- Existing disambiguation tests stay green.
- No regressions in non-name rendering paths.

## Risks and Mitigations
- Risk: accidental change to hyphenated given-name behavior.
  - Mitigation: explicit test with `initialize_with_hyphen = Some(false)` and default behavior.
- Risk: whitespace normalization could alter legacy edge cases.
  - Mitigation: constrain normalization to initialized-output assembly, not raw input mutation.

## Suggested Execution Order
1. Write failing tests.
2. Implement formatter update in `format_single_name`.
3. Re-run targeted tests.
4. Run full Rust checks.
5. Land with conventional commit once green.
