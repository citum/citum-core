# Bean Resolution Wave (2026-03-09)

## Purpose

Resolve four open beans whose status no longer matched current repository state,
while landing the one remaining migrate fix that was still missing.

Related beans:
- `csl26-bpuw`
- `csl26-6i1c`
- `csl26-9a89`
- `csl26-ctw8`

Residual portfolio work remains tracked by
[`docs/specs/ENGINE_MIGRATE_COEVOLUTION_WAVE.md`](./../specs/ENGINE_MIGRATE_COEVOLUTION_WAVE.md).

## Evaluation

### `csl26-bpuw`

This bean is already satisfied on `main`.

Evidence:
- `crates/citum-engine/src/values/variable.rs` already wires `archive` and
  `archive_location`.
- `crates/citum-engine/tests/domain_fixtures.rs` already contains the focused
  humanities-note regression test for manuscript, interview, and
  personal-communication rendering.
- Current branch verification passed:
  - `cargo test -p citum-engine test_humanities_note_fixture_preserves_archive_and_interview_fields -- --exact --nocapture`
  - `cargo test -p citum-engine chicago_notes -- --nocapture`

Conclusion: close as completed and archive. No new Chicago code was required in
this PR.

### `csl26-6i1c`

This bean duplicates `csl26-bpuw`.

Evidence:
- The body is a shortened restatement of the same humanities-note fixture
  recovery problem.
- It adds no independent acceptance criteria or separate implementation path.

Conclusion: scrap as duplicate and archive.

### `csl26-9a89`

This bean is stale-open.

Evidence:
- Its completed-work list matches functionality already present on `main`,
  including entry suffix support, delimiter detection refinements, and issue
  suppression infrastructure.
- The remaining unchecked items are broader migrate/fidelity work rather than a
  single bounded bug fix.

Conclusion: close as completed for the concrete rendering fixes already landed,
and treat any remaining broad work as part of the active co-evolution wave.

### `csl26-ctw8`

This bean still required code.

Gap before this PR:
- `crates/citum-migrate/src/upsampler.rs` discarded node-local
  `strip_periods` metadata for legacy `Text` and `Label` nodes.

Fix landed in this PR:
- preserve `strip_periods` in the upsampler
- carry label-driven stripping into compiled number, contributor, and locator
  template output where the engine already supports it
- add migrate tests that prove preservation at both upsample and compiled
  template layers

Conclusion: complete and archive.
