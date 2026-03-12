---
# csl26-qyrq
title: Make oracle scoring case-aware and roll out text-case to all styles
status: todo
type: feature
priority: high
created_at: 2025-07-14T12:00:00Z
updated_at: 2025-07-14T12:00:00Z
---

Follow-up to `csl26-zc4m` (title text-case semantics implementation).

Context: PR #337 landed the text-case engine support. This bean tracks
making the oracle scoring pipeline case-sensitive and rolling out
`text_case` configuration to all styles.

## Key Finding

`oracle-utils.js` applies `.toLowerCase()` before all comparisons, so
case differences have been invisible to fidelity scores. This means
current baselines silently accept casing mismatches, masking the impact
of missing `text_case` configuration in style YAML files.

## Plan

### Phase 1 — Oracle case-awareness

- Add `--case-sensitive` flag to oracle scripts
- Add case-mismatch counts as a separate metric in `report-core` output
- Run baseline snapshot with the new metric to establish the current gap

### Phase 2 — Migration text-case extraction

- Wire CSL `text-case` extraction into `upsampler.rs` for variables
  (currently discarded)
- Emit `TitleRendering` config in migration output so styles carry
  casing intent from their CSL sources

### Phase 3 — Style audit and updates

- Categorize 157 styles by expected text-case from CSL sources
- Batch-add `text_case` to style YAML configs based on the audit

### Phase 4 — Case-sensitive scoring gate

- Make case-sensitive the default scoring mode
- Regenerate baselines with case-sensitive comparisons
- Add verification-policy divergences for intentional differences

## Todos

- [ ] Add `--case-sensitive` flag to oracle scripts
- [ ] Add case-mismatch metric to `report-core`
- [ ] Wire `text-case` extraction in `upsampler.rs`
- [ ] Audit 157 styles for expected text-case
- [ ] Batch-update style YAML configs with `text_case`
- [ ] Regenerate baselines case-sensitively
- [ ] Document verification-policy divergences
