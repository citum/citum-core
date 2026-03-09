# Architectural Soundness Assessment — Citum

**Date:** 2026-02-25
**Context:** Retrospective before final push; review of original bdarcus/csln ideas as implemented and validated in csl26

---

## Verdict

The ideas are sound. The empirical record is not ambiguous:

- 10/10 top parent styles at 100% strict fidelity
- 100 styles in catalog; ~58% dependent corpus coverage
- The alternative approach (XML compiler for templates) achieved 0% bibliography fidelity vs 100% for hand-authored YAML

The original 2023 schema intuitions were correct. LLM-assisted implementation validated them at a scale the original work could not reach alone.

---

## What Has Been Validated

| Idea | Evidence |
|---|---|
| Flat declarative templates | 10/10 top styles at 100% strict match |
| Type overrides replace `choose/if/else` | Hand-authored APA: 11 template components vs 126 CSL `choose` blocks |
| Presets for common configurations | 58-style wave uses date/title/contributor presets with no fidelity loss |
| Serde as schema source of truth | JSON schema generated from Rust types; round-trip tests pass |
| Processor stays dumb, style declares behavior | Known gaps addressed by adding style fields (`volume_pages_delimiter`, `doi_prefix`), not processor logic |
| Hybrid migration strategy (XML options + output-driven templates) | `template-inferrer.js`: 95–97% confidence across 6 styles; XML compiler at 0% bib fidelity |
| Oracle verification loop | CI gate prevents regressions; strict 12-scenario set |

The decisive test was the migration strategy head-to-head: the XML compiler approach translated procedural CSL 1.0 logic into procedural Rust, not into declarative templates — and the output showed it. Hand-authoring with YAML overrides was the only approach that worked. See [MIGRATION_STRATEGY_ANALYSIS.md](./MIGRATION_STRATEGY_ANALYSIS.md).

---

## Remaining Gaps (Scope, Not Design Flaws)

These are well-understood open scope items, not architectural problems:

### 1. Note styles — `position` condition not implemented
- Requires stateful citation graph (ibid, subsequent, first)
- Architecturally understood; deferred to Phase 3
- Affects ~19% of corpus (Chicago Notes, OSCOLA, MHRA)
- Tracked in ROADMAP.md Phase 3

### 2. Sorting verification
- Sort templates are designed (PRIOR_ART.md Issue #61)
- Not yet systematically tested against oracle across styles
- Affects all bibliography styles
- **Not yet tracked as a bean** → see `csl26-sort-verify`

### 3. Locale-specific template layouts
- `bibliography.locales[].template` pattern proposed in PRIOR_ART.md
- Not yet implemented
- Needed for Japanese/CJK and other mixed-language bibliography layouts
- **Not yet tracked as a bean** → see `csl26-locale-templates`

### 4. Disambiguation at full CSL 1.0 fidelity
- Year-suffix and basic author disambiguation work
- Full CSL 1.0 (add-names, add-givenname, choose-based) not complete
- Tracked in [DISAMBIGUATION_IMPLEMENTATION_PLAN.md](./DISAMBIGUATION_IMPLEMENTATION_PLAN.md)

### 5. Legal citation support
- LEGAL_CITATIONS.md design document exists
- Jurisdiction hierarchy, parallel citations, position extensions not modeled in type system
- Queued (OSCOLA, Bluebook)

### 6. Output format pluggability
- Current: plain text, HTML, Djot, LaTeX, Typst
- Still missing: RTF
- `Renderer` trait is implemented and pluggable across the current formats
- Completed under archived bean `csl26-ismq`; follow-on rendering edge cases
  continue in narrower beans

### 7. Interactive/incremental mode
- Architecture is batch-first
- JSON server mode (design principle 2) not yet built
- Tracked under ROADMAP Phase 4

### 8. Test fixture coverage for rare types
- 28-item fixture covers 12 reference types
- Legal, patent, dataset, entry types underrepresented
- Template inferrer confidence degrades for rare type-specific behavior

---

## One Honest Wart: `InputReference` and `deny_unknown_fields`

**What it is:** Style and locale types use `deny_unknown_fields` to catch typos at parse time. `InputReference` variants cannot — internally-tagged enum dispatch replays the tag field into the inner struct's deserializer, making `deny_unknown_fields` incompatible with serde. Unknown fields on reference types silently produce `None` and surface as missing data at render time rather than a parse error.

**Why it matters for stabilization:** A caller who passes a reference with a mistyped field name (e.g., `publischer` instead of `publisher`) gets no error — the field is silently ignored and the output is wrong. This is hard to debug.

**Options before spec stabilization:**
1. Post-deserialize validation pass that checks for unexpected `None` values on required fields (e.g., `publisher` for `book` type)
2. A `#[serde(deny_unknown_fields)]`-equivalent via a custom `Deserialize` impl on the enum (complex but possible)
3. Accept it as documented and add a `csln validate` subcommand that users can run explicitly

This is not a design flaw. It is a Serde limitation with internally-tagged enums. The trade-off is documented in DESIGN_PRINCIPLES.md Section 3. The question is whether to mitigate it before calling the schema stable.

Tracked as bean `csl26-inputref-wart`.

---

## References

- [MIGRATION_STRATEGY_ANALYSIS.md](./MIGRATION_STRATEGY_ANALYSIS.md)
- [DESIGN_PRINCIPLES.md](./DESIGN_PRINCIPLES.md) — Section 3 (deny_unknown_fields trade-off)
- [PRIOR_ART.md](./PRIOR_ART.md)
- [ROADMAP.md](./ROADMAP.md)
- [DISAMBIGUATION_IMPLEMENTATION_PLAN.md](./DISAMBIGUATION_IMPLEMENTATION_PLAN.md)
