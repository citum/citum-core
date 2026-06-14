# Migrate Fidelity Tail — Locus Classification

- **Date:** 2026-06-14
- **Bean:** `csl26-cvlm`
- **Instrument:** `node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610`
- **Measured headline:** 67/100 styles at ≥90% combined strict fidelity (commit `2ac72297`), 33 sub-90, 0 hard errors.

## Why this audit exists

The assertion that *"remaining fidelity gaps are engine-level, not
converter-level"* is stated as settled fact in `crates/citum-migrate/CLAUDE.md`
and echoed in [OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md](../../specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md)
and the [2026-06-11](2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md) /
[2026-06-14 order-aware fitness](2026-06-14_MIGRATE_ORDER_AWARE_FITNESS_NEGATIVE.md)
audits. The 2026-06-14 audit's own #1 next lever was to **classify the sub-90
tail into engine / converter / genuinely-hard** before further work. That
classification had never been run. This audit runs it.

## Verdict: the blanket "engine-level, not converter-level" claim is stale and wrong

Two independent lines of evidence:

1. **The cited engine evidence is already resolved.** Both prior audits name
   `csl26-y4o7` (once-only variable consumption) as *the* live engine gap. It
   was **completed 2026-06-12** — before the 2026-06-14 audit that still cites it.
   The three engine near-misses tracked in session memory (`nature`,
   `chicago-author-date`, `cell`) now pass **fully** via the oracle.

2. **The current sub-90 tail is converter-dominated.** Every style sampled
   across all five style classes fails for converter-level reasons — missing or
   wrong template data — not "the engine renders correct template data wrong."

## Sampled classification

| Style | Class | Combined | Locus | Symptom |
|---|---|---|---|---|
| american-mathematical-society-label | label | 43→45 | **converter** (now fixed) + engine residual | `citation-label` dropped → empty `[]`; after fix, residual double-bracket / trigraph length |
| bibtex | label | 71 | converter | label citation rendered as author-date key |
| din-1505-2-alphanumeric | label | 74 | converter (now fixed) | label dropped; bib 36/38 after fix |
| bio-protocol | numeric | 66 | converter | numeric style migrated as author-date (`[16]` → `(Kuhn, 1962)`) |
| journal-of-advertising-research | author-date | 78 | converter | titles double-quoted `““…””`; spurious trailing URL/accessed |
| china-information | note | 86 | converter | leaked literal `Entry encyclopedia. in.`; given-name form; dropped web access group |
| early-medieval-europe | note | 45 | converter | note citation component order + affixes diverge; double-quote |
| springer-physics-author-date | author-date | 85 | converter + **genuinely-hard** | et-al over-truncation; `legal_case` template |

**All five style classes have sub-90 members; every sampled failure is
converter-level** (with `legal_case` the only genuinely-hard residue). None is
the assertion's "correct template, wrong render" mode.

## Fixed in this pass (converter-level, with regression tests)

The label cluster was fully broken — `citation-label` was silently dropped, so
all 3 label-class styles in the corpus failed. Two root causes, both converter:

1. **`map_variable_to_number` had no `CitationLabel` arm**
   (`crates/citum-migrate/src/template_compiler/node_compiler.rs`) — `<text
   variable="citation-label"/>` compiled to `None` and was dropped from both
   citation and bibliography templates. The engine, schema, IR, and variable
   mapping all already support `citation-label`; only this arm was missing.

2. **`detect_processing_mode` never returned `Processing::Label`**
   (`crates/citum-migrate/src/options_extractor/processing.rs`) — the engine
   only emits trigraph labels under `Processing::Label`, so even a correct
   template rendered empty. Added detection keyed on a `citation-label` text
   node in the citation layout.

After both fixes, label styles **render labels** (`[]` → `[Kuh62]`); din-1505
bibliography recovered to 36/38.

## Why the headline stayed at 67/100

Correct converter fixes did **not** move the pass-count headline (67 → 67, only
AMS-label +1.7). This is the real, durable lesson — and it is *not* the
"engine-level ceiling" framing:

- The pass count is binary at a 0.60 per-item threshold. Sub-90 styles carry
  **multiple compounding defects**. Fixing one (label now renders) leaves others
  (double-bracket wrap, trigraph length, multi-cite ordering) that still hold
  the item below threshold.
- So the correct model is **"compounding converter defects under a binary
  threshold,"** not "an engine ceiling." Converter fixes are necessary and
  correct even when they don't move the headline; the headline moves only when
  *all* of a style's defects clear.

## Follow-up beans (all parented to `csl26-vmcr`)

| Bean | Cluster |
|---|---|
| `csl26-tzer` | label double-bracket wrap + trigraph length (engine/config residual) |
| `csl26-dc1d` | numeric style migrated as author-date (bio-protocol) |
| `csl26-c2um` | titles double-quoted in migrated templates |
| `csl26-ahxh` | note-class citation component order + affixes |
| `csl26-ya9b` | spurious URL/accessed + leaked term/type text in bibliography |

## Recommendation

Treat the migrate fidelity tail as **ordinary converter bug-fixing**, not a
blocked engine problem. The "engine-level ceiling" language in
`crates/citum-migrate/CLAUDE.md` and the synthesis spec should be downgraded to
"compounding converter defects" and point here.
