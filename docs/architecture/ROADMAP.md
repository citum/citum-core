# Citum Project Roadmap

**Last updated:** 2026-02-26
**Purpose:** Strategic plan tracking project maturity, phases, and risks

**Canonical metrics source of truth:**
- `docs/TIER_STATUS.md` for style-level strict oracle status
- `scripts/report-data/core-quality-baseline.json` for portfolio baseline gates
- `docs/compat.html` for published compatibility snapshot

## Current State Matrix

### Foundation (Complete ✅)

| Component | Status | Coverage | Notes |
|-----------|--------|----------|-------|
| CSL 1.0 Parser | ✅ Complete | 2,844 styles | Full XML parsing capability |
| Citum Schema | ✅ Complete | - | Style, Template, Options, Locale types |
| Type System | ✅ Designed | - | Hybrid model with 4-factor test policy |
| EDTF Dates | ✅ Operational | - | Range/uncertainty handling shipped; historical-era rendering specified in `docs/specs/EDTF_HISTORICAL_ERA_RENDERING.md`; broader era policy remains follow-up |

### Migration Pipeline (Operational ✅)

| Component | Status | Accuracy | Notes |
|-----------|--------|----------|-------|
| XML Options Extraction | ✅ Operational | 87-100% citations | ~2,500 lines, DO NOT TOUCH |
| Output-Driven Templates | ✅ Validated | 95-97% confidence | Tested on 6 styles |
| LLM Hand-Authoring | ✅ Operational | See `docs/TIER_STATUS.md` | Production styles converted and maintained via style-evolve |
| Oracle Verification | ✅ Complete | - | Structured diff, batch aggregator |

### Processor (Format-Specific Readiness)

| Format | Citations | Bibliography | Blockers |
|--------|-----------|--------------|----------|
| Author-Date | See `docs/TIER_STATUS.md` | See `docs/TIER_STATUS.md` | Long-tail variation cleanup |
| Numeric | See `docs/TIER_STATUS.md` | See `docs/TIER_STATUS.md` | Residual long-tail outliers |
| Note | See `docs/TIER_STATUS.md` | See `docs/TIER_STATUS.md` | Position-sensitive edge cases |

**Output Formats:** Plain text ✅, HTML ✅, Djot ✅

### Tooling (Optimized ✅)

| Tool | Status | Impact | Notes |
|------|--------|--------|-------|
| Oracle Verification | ✅ Complete | 88% token reduction | Caching, structured diff |
| Workflow Scripts | ✅ Complete | 4 phases validated | prep-migration.sh, workflow-test.sh |
| /style-evolve Workflow | ✅ Complete | Repeatable migrate/upgrade loops | `styleauthor` retained as legacy alias |
| Benchmarking | ✅ Available | Opt-in for hot paths | rendering, formats benchmarks |

## Phase Plan

### Phase 1: Author-Date Quality Refinement (Current)
**Target:** Maintain strict parity for high-impact author-date parents while reducing long-tail drift
**Duration:** 2-3 weeks
**Approach:** /style-evolve iteration loops

**Styles:**
1. APA
2. Elsevier Harvard
3. Chicago Author-Date variants
4. Springer Basic Author-Date variants

**Success Criteria:**
- No regressions in strict oracle results for already-passing styles
- Reduced repeated long-tail mismatch clusters
- Common failure patterns documented
- Workflow optimization insights captured

**Risks:**
- LLM budget overruns if processor features missing
- Variation in style complexity (APA success may not predict others)

### Phase 2: Numeric Style Features (Next)
**Target:** Close remaining numeric outliers after top-tier parity
**Duration:** 3-4 weeks
**Approach:** Feature implementation + /style-evolve iteration

**Prerequisites:**
- Year-positioning and ordering parity fixes
- Citation numbering system
- Superscript support
- Sorting templates

**Styles:**
1. Elsevier With-Titles (672 dep)
2. IEEE (176 dep)
3. Elsevier Vancouver (502 dep)
4. American Medical Association (293 dep)
5. Springer Vancouver Brackets (472 dep)
6. Springer Basic Brackets (352 dep)

**Success Criteria:**
- No regressions in top-tier numeric styles
- Reduced numeric outlier mismatch clusters
- Citation numbering works reliably
- Sorting matches citeproc-js output

**Estimated Effort:** 500-800 lines new code

**Risks:**
- Numeric features more complex than anticipated
- Sorting edge cases (same author, same year, etc.)
- Timeline delay could push 60% coverage goal

### Phase 3: Note Styles (Deferred)
**Target:** 542 note styles (~19% corpus)
**Duration:** TBD
**Approach:** Feature implementation after numeric validation

**Prerequisites:**
- Position tracking (ibid, subsequent, first)
- Note-specific formatting (no parentheses, different delimiters)
- Disambiguation in notes context

**Styles:**
1. Chicago Notes
2. OSCOLA (legal)
3. MHRA

**Decision Point:** Reassess after Phase 2 complete to validate approach repeatability

### Phase 4: Production Readiness (Future)
**Target:** Beta-ready for Zotero/Pandoc integration
**Duration:** TBD

**Features:**
- WASM build (browser/plugin integration)
- JSON server mode (minimize startup latency)
- API stability (versioned schema, migration guide)
- Performance optimization (benchmarks, profiling)

**Prerequisites:**
- 10+ parent styles at 80%+ match
- Comprehensive test coverage
- Documentation complete

## Key Metrics Dashboard

| Metric | Current | Phase 1 Target | Phase 2 Target | Notes |
|--------|---------|----------------|----------------|-------|
| Top-10 strict parity | See `docs/TIER_STATUS.md` | Maintain parity | Maintain parity | Single source: `docs/TIER_STATUS.md` |
| Portfolio fidelity/SQI | See `scripts/report-data/core-quality-baseline.json` | Increase threshold attainment count | Increase threshold attainment count | Evaluated via `report-core` + `check-core-quality` |
| Long-tail outliers | See latest core report | Reduce repeated citation clusters | Reduce repeated bibliography clusters | Use `scripts/analyze-migration-gaps.js` |
| Bean hygiene | See `.beans/` audit | Normalize status taxonomy | Keep duplicates at zero for active migrate tasks | Enforced by hygiene script |

## Risk Register

### High Priority Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Numeric timeline delay | 60% coverage goal delayed | Medium | Focus Phase 1 on author-date only, prove repeatability first |
| LLM budget overruns | Extended iteration time | Medium | Document patterns, optimize workflow, use @builder for processor features |
| Perception gap | Marketing as "CSL replacement" but note styles deferred | Medium | Transparent roadmap, focus on 60% corpus coverage first |
| Springer regression | Citation quality drop | Low | Tracked, isolated to single style, fix in Phase 1 |

### Medium Priority Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Workflow optimization not transferable | APA success doesn't predict other styles | Medium | Test against diverse styles (Elsevier, Chicago, Springer) |
| Numeric features more complex | 500-800 line estimate too low | Low | Incremental implementation, cargo test guard |
| Note style complexity | Position tracking harder than anticipated | Low | Defer to Phase 3, reassess after numeric validation |

## Workflow Optimization Notes

### What Works (Validated)
- 5-phase style-evolve workflow (research → author → test → evolve → verify)
- Structured oracle comparison (component-level diff)
- Iterative refinement with processor evolution
- Reusable pattern capture in common-patterns.yaml

### What Needs Improvement
- Numeric long-tail style support
- Workflow budget optimization (actual time vs 18min target)
- Failure pattern documentation (systematic categorization)
- Cross-style consistency (delimiter variations, volume/issue formatting)

### Common Failure Modes
1. **Year positioning** - Numeric styles show year in wrong position
2. **Volume/issue spacing** - "2, (2)" vs "2(2)" inconsistency
3. **Editor labels** - Capitalization and punctuation variations
4. **Page delimiters** - Comma vs colon varies by style
5. **DOI suppression** - Some styles don't output DOI

## Decision Log

### 2026-02-25: Architectural Soundness Validated
**Decision:** Core architectural ideas (flat declarative templates, type overrides, serde-as-truth, explicit options) are empirically validated at production scale.
**Rationale:** 10/10 top parent styles at 100% strict fidelity; XML compiler alternative achieved 0% bibliography fidelity. Remaining gaps are scope (note styles, sorting, locale templates), not design flaws. One known wart: `InputReference` + `deny_unknown_fields` incompatibility should be mitigated before spec stabilization.
**Refs:** [ARCHITECTURAL_SOUNDNESS_2026-02-25.md](./ARCHITECTURAL_SOUNDNESS_2026-02-25.md)

### 2026-02-15: Hybrid Migration Strategy Validated
**Decision:** Use XML options + output-driven templates + LLM authoring
**Rationale:** XML excels at options (87-100% cit), fails at templates (0% bib). LLM authoring achieves 14/15 bibliography for APA.
**Refs:** ./architecture/MIGRATION_STRATEGY_ANALYSIS.md, bean csl26-m3lb

### 2026-02-08: Defer Note Styles to Phase 3
**Decision:** Focus author-date (40% corpus) then numeric (20% corpus) before note (2% corpus)
**Rationale:** Maximize coverage with proven approach before tackling position tracking complexity.
**Refs:** ./../reference/STYLE_PRIORITY.md, bean csl26-5t6s

### 2026-02-08: Type System Architecture Finalized
**Decision:** Hybrid model (structural for academic, flat for legal/domain-specific)
**Rationale:** Balances data efficiency with style clarity using 4-factor test.
**Refs:** ./architecture/design/TYPE_SYSTEM_ARCHITECTURE.md, ./architecture/design/TYPE_ADDITION_POLICY.md

## References

- **Architecture Docs:** ./architecture/
- **Bean Tracker:** ../.beans/
- **Style Priority:** ./../reference/STYLE_PRIORITY.md
- **Migration Analysis:** ./architecture/MIGRATION_STRATEGY_ANALYSIS.md
- **Workflow Guide:** ./../guides/RENDERING_WORKFLOW.md
