# Migration Approach — Strategic Review

- **Date:** 2026-07-17
- **Bean:** `csl26-bv8w`
- **Question:** Deterministic CSL→Citum conversion has plateaued and high-impact
  styles now rely on expensive LLM/human tuning (PR #1061 took over a day of
  agent work). Does this indicate something was missed in the migration
  approach or in Citum's design itself — or is hand-tuning a legitimate
  learning loop that improves the Rust pipeline?
- **Evidence base:** the 2026-06 migrate audit series
  ([baseline](2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md),
  [wave outcome](2026-06-11_MIGRATE_IMPROVEMENT_WAVE_OUTCOME.md),
  [order-aware fitness negative](2026-06-14_MIGRATE_ORDER_AWARE_FITNESS_NEGATIVE.md),
  [locus classification](2026-06-14_MIGRATE_FIDELITY_LOCUS_CLASSIFICATION.md)),
  the [2026-07-06 migrate crate review](2026-07-06_CITUM_MIGRATE_REVIEW.md),
  [MIGRATION_STRATEGY_ANALYSIS.md](../MIGRATION_STRATEGY_ANALYSIS.md),
  [OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md](../../specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md),
  and a line-level decomposition of PR #1061.

## Verdict

**Nothing fundamental was missed.** The plateau is the structural price of the
semantic uplift Citum deliberately chose (procedural → declarative/typed); the
hybrid strategy already in place — hand-author top parents, converter as
evidence, seed, and long-tail automation — is the correct response, and the
hand-tuning hypothesis is confirmed: tuning demonstrably feeds converter,
engine, and schema fixes and is already institutionalized as the operating
model. The leverage going forward is **extending the alias economy to hidden
families among independent styles** and **making tuning cheaper**, not making
conversion smarter.

## 1. The plateau is measured, structural, and honestly recorded

The repo has interrogated its own approach with unusual rigor:

- **Seeded random-100 scorecard:** 43/100 → 52 → 53 → 67/100 styles at ≥90%
  combined strict fidelity across the June waves. Distribution is high and
  left-skewed: median style 94.7%, p90 = 100%, p10 = 74. Automatic conversion
  is, by decompiler standards, remarkably good.
- **Rising cost-per-point:** the early deterministic fixes delivered 43 → 52
  cheaply; the most expensive later item delivered +1. The wave was stopped on
  economics, with the decision recorded.
- **Smarter search is a proven dead end:** the order-aware fitness experiment
  was implemented, measured (67 → 67, net −1.7 churn), reverted, and recorded
  as structurally futile — the pass-count headline is decoupled from the
  scoring gradient by design.
- **The tail is compounding converter bugs, not an engine ceiling:** the locus
  classification overturned the earlier "engine-level" framing. Sub-90 styles
  carry several independent converter defects each, so correct single fixes
  are headline-invisible under the binary 0.60 threshold.

## 2. Why deterministic conversion plateaus (the structural argument)

CSL 1.0 is procedural: macros, `choose/if` trees, groups with *implicit*
empty-suppression. Citum is declarative, typed, and explicit by design
([DESIGN_PRINCIPLES.md](../DESIGN_PRINCIPLES.md)). Conversion between the two
is therefore **decompilation** — recovering intent from a procedural encoding.
Decompilers reliably achieve "correct but non-idiomatic"; idiomatic output
requires intent the source does not carry.

The sharper form of the point: **if a total deterministic high-fidelity
mapping existed, Citum would be structurally equivalent to CSL** — CSL-in-YAML
— and would inherit the maintainability properties the project exists to
escape. The migration difficulty is the price of the design goal, not evidence
against it. There was no missed algorithm.

## 3. PR #1061's cost is mis-attributed: it was capability construction

Line-level decomposition of the ~12,550 additions:

| Bucket | Approx. lines | Nature |
|---|---|---|
| GB/T style YAML family (4 files) | ~1,700 | The actual "tuned styles" |
| CSL-M test fixture | ~600 | Corpus asset |
| CSL-M parsing (`csl-legacy`) | ~400 | Durable dialect support |
| CSL-M layout support (`citum-migrate`) | ~600 | Durable converter capability |
| Engine date/number/i18n fidelity | ~650 | Durable engine fixes |
| GB/T reference data model (`citum-schema-data`) | ~500+ | Durable schema capability |
| zh-CN locale + oracle locale support | ~350 | Durable multilingual asset |
| JSON schemas, specs (TEMPLATE_V3, MULTILINGUAL, REFERENCE_IDENTIFIERS), tooling | ~700 | Durable docs/tooling |

Roughly **85% of the PR is one-time, reusable capability**, not style tuning.
GB/T 7714—2025 was close to a worst-case test: a CSL dialect (CSL-M) the
pipeline did not support, a script it did not render, reference types it did
not model, and a national-standard revision with no existing oracle target.
Judging steady-state tuning economics by this PR is judging a compiler by its
hardest port. The next CSL-M or Chinese-standard style pays none of these
costs again.

## 4. The learning loop is real and already codified

- `style-tune`'s failure classification routes mismatches to
  `migration-artifact` (fix the converter seed) and `processor-defect`
  (escalate to the Rust workflow) and explicitly forbids cycling YAML to
  compensate for either.
- [ENGINE_MIGRATE_COEVOLUTION_WAVE.md](../../specs/ENGINE_MIGRATE_COEVOLUTION_WAVE.md)
  formalizes "convert repeated style-fidelity failures into shared migrate and
  engine fixes before residual style-local cleanup."
- The locus-classification pass itself — a product of tuning-adjacent
  investigation — found and fixed two real converter bugs (dropped
  `citation-label` mapping; missing `Processing::Label` detection).

So the answer to the review question's second half is yes: hand-tuning is not
merely a stopgap; it is the discovery mechanism the deterministic pipeline
feeds on, and the feedback path is contractual, not aspirational.

## 5. Honest counterpoints

- **Oracle-as-target imports citeproc-js quirks.** Byte-parity with
  citeproc-js means both inference and tuning can enshrine oracle bugs.
  Hand-tuning's style-guide authority basis is the mitigation; pure conversion
  has no equivalent check. A target choice, not a flaw — but keep it explicit.
- **Compensating errors cut both ways.** Tuning can bake engine-bug
  workarounds into YAML just as inference can. The engine-review audit series
  is the counterweight; keep it running.
- **The dual hard gates (100% fidelity + clean SQI) are the cost driver.**
  Right for embedded-core parents; they must not silently become the bar for
  the long tail.

## 6. Where the leverage actually is

1. **Extend the alias economy to hidden families among independent styles.**
   CSL *dependent* styles are already handled at zero migration cost: they
   become registry aliases
   ([STYLE_ALIASING.md](../../specs/STYLE_ALIASING.md): 7,987 dependents alias
   ~300 parents; the top 10 parents cover 60%). That economy is banked. The
   remaining per-style cost lives in the ~2k+ *independent* styles the
   random-100 corpus samples — each currently synthesized from scratch, even
   when it is a near-clone of an already-tuned parent. The leverage is family
   detection: classify an independent style against tuned/embedded parents and
   emit a small `extends` delta instead of full synthesis. The machinery
   exists (`base_detector`, lineage/wrapper emission, `template_diff`), and
   `csl26-b4h2` already proposes scripted discovery of hidden parent-style
   alias candidates. **Gap: nobody has measured what fraction of the
   independent corpus is expressible as a small delta over a tuned parent at
   fidelity ≥ its current synthesized result.** That fraction — not the
   style-count-weighted random-100 headline, which usage-weighted coverage
   already far exceeds — is the true remaining cost model.
   Follow-up bean: `csl26-7iiu`. The compat dashboard should also surface
   this inheritance picture (parent families, aliases, deltas) instead of a
   flat per-style list: `csl26-zik7`.
2. **Instrument tuning cost.** If the flywheel works, cost per tuned style
   should fall over time. Record tokens/wall-time per tune pass in the
   `style-tune` output contract so the trend is visible instead of anecdotal.
   Follow-up bean: `csl26-m2t1`.
3. **Bank every tuned style as gold evidence.** A tuned style can outrank
   citeproc-js output, but only through an explicit mechanism, never as a
   blanket claim: the tune loop requires a primary authority basis (publisher
   guide or style manual), and any intentional divergence from the oracle must
   be registered in `scripts/report-data/verification-policy.yaml` as a
   `divergences:` entry with scope and justification — the default authority
   remains citeproc-js for everything unregistered. "Gold evidence" is then
   concrete: each tuned style contributes (a) its YAML and rendered output as
   converter regression targets and (b) its registered divergence entries, so
   converter improvements are measured against human-verified styles, not
   only against the oracle.
4. **Reconcile stale goalposts.** The synthesis spec still carries the
   ">80/100" acceptance goal that the 2026-06-11 stop decision effectively
   retired, and the XML-compiler removal gate (`csl26-hxhx`) remains open.
   Re-scope the 80/100 line as a long-horizon aspiration, not a gate.
   Follow-up bean: `csl26-nzjo`.
5. **Do not fund further conversion cleverness.** Treat the sub-90 tail as
   ordinary bug-fixing, done opportunistically when tuning or cohort waves
   surface a cluster — the coevolution-wave pattern, already in force.
