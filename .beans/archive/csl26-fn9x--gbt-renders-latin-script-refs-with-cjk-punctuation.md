---
# csl26-fn9x
title: GB/T renders Latin-script refs with CJK punctuation
status: completed
type: bug
priority: high
tags:
    - fidelity
    - multilingual
    - style
created_at: 2026-07-17T21:06:17Z
updated_at: 2026-07-17T23:02:51Z
---

citum render refs --style gb-t-7714-2025-numeric -b tests/fixtures/test-items-library/gb-t-7714-2025.json renders English refs with full-width punctuation (Chichester：John Wiley & Sons，2020：35) where GB/T practice uses Latin punctuation for Latin-script references. Root cause: the CSL-M source style hardcodes full-width delimiters, citeproc-js reproduces them, and our fidelity gate uses citeproc-js as sole authority (verification-policy: gb-t-7714-2025-numeric, authority: citeproc-js) - so byte-parity (198/203 in PR 1064) is satisfied while the output is non-conformant with the standard itself. Fix belongs in the Citum GB/T styles (per-item-language punctuation, see docs/specs/MULTILINGUAL.md), with the intentional oracle divergence registered in scripts/report-data/verification-policy.yaml divergences. Same failure class as the tier-0 alias negative result: verification proxy weaker than the real requirement.

## Implementation Plan (see /home/bruce/.claude/plans/complete-bean-csl26-fn9x-via-gentle-lark.md)

- [x] Engine: ScriptConfig.punctuation + PunctuationStyle enum (citum-schema-style)
- [x] Engine: remap_to_latin_punctuation + wants_latin_punctuation, wired at 3 insertion points
- [x] Style: opt-in options.multilingual.scripts.latin.punctuation on gb-t-7714-2025-base.yaml
- [x] Oracle: div-010 detector in oracle-divergences.js + verification-policy.yaml entry
- [x] Oracle: div-010 unit tests plus determineBenchmarkStatus adjusted-precedence test
- [x] Rust behavior tests in multilingual.rs: Latin remap, CJK control, opt-in gate
- [x] Docs: audit record plus MULTILINGUAL.md section 3.2a Active
- [x] Schema regen (just schema-gen)
- [x] Verified render output, before/after oracle diff, pre-commit gate (2053/2053 nextest, fmt+clippy clean)

## Summary of Changes

Added `options.multilingual.scripts.latin.punctuation: latin` (engine-level, opt-in) so Latin-script GB/T references render Latin half-width punctuation (`: , ( )`) instead of the style's CJK-facing full-width delimiters (full-width colon/comma/parens), while CJK and Cyrillic items are unaffected. Enabled once on the shared gb-t-7714-2025-base.yaml, inherited by all three heads.

Required three insertion points in citum-engine, since full-width punctuation enters rendered output from three independent places: render::component (per-component value/prefix/suffix/wrap), render::citation::citation_to_string_with_format (citation-section wrap/delimiter), and processor::citation::apply_spec_wrap_and_affixes (outermost citation-spec wrap). Design documented in docs/specs/MULTILINGUAL.md section 3.2a (revised after external review to narrow scope claims and lead with the positive-evidence-only rule).

Registered div-010 in the oracle-adjustment system (scripts/lib/oracle-divergences.js plus verification-policy.yaml) to mask the resulting intentional citeproc-js divergence for Latin-script items only, gated on a punctuation-only-delta check.

Found and fixed two pre-existing infrastructure bugs while verifying: (1) buildAdjustedOracleResult never recomputed the bibliography passed/failed aggregate from adjusted entries (only citations were recomputed) -- silently neutered every bibliography-scope divergence, not just this one. (2) determineBenchmarkStatus (the actual fidelity gate) read raw counts, never .adjusted -- meaning no registered divergence had ever actually kept any style's gate green before this fix. Both fixed; the fix is provably monotonic (adjusted match count is always >= raw), confirmed no status flips across all 5 currently-gated styles.

Verified via direct before/after oracle diff (clean baseline build vs this branch, same fixture): 27 new raw deltas introduced by the punctuation remap, all fully masked, zero unmasked regressions. Adjusted failure count improved (29 to 18). The min_pass_rate 1.0 gate was already failing before this change (173/203 raw, not 198/203 as originally assumed) for 18 pre-existing, unrelated reasons (missing components, ordering gaps on anonymous-author entries) -- filed as follow-up bean csl26-d3hs. Fixing those is out of scope here.

Tests: 3 new Rust behavior tests (multilingual.rs), 5 new plus 1 new JS unit tests (oracle.test.js, report-core.test.js). Full workspace cargo nextest: 2053/2053 passed. fmt and clippy clean.
