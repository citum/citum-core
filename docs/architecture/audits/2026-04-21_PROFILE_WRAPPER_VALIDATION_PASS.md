# Profile Wrapper Validation Pass

**Date:** 2026-04-21
**Bean:** `csl26-nrkn`
**Related:** `docs/specs/STYLE_TAXONOMY.md`, `docs/architecture/PRESET_WRAPPER_AUTHORITY_PASS_2026-04-19.md`, `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`

## Summary

This pass re-checked the embedded publisher/profile styles that had been
directionally grouped during the style-taxonomy rename work.

The governing decision rule stayed the same:

1. publisher or journal guide
2. publisher house rules
3. named parent-style manual or standards reference
4. CSL/template-link evidence
5. current Citum YAML structure

Two outcomes matter:

- `profile` can stay the correct taxonomy even when the current public YAML is
  still self-contained, as long as the parent-plus-deltas relationship is
  guide-backed.
- No new `extends:` conversion was landed in this pass. The only plausible
  existing-base conversion (`springer-basic-brackets` → `springer-basic-author-date`)
  still requires almost the whole file under the current merge contract, so it
  does not yet meet the tightened meaning of a thin profile. The blocker here
  is not uncertainty about parentage; it is that bibliography/type-variant
  deltas live in replace-whole structures, so a child style cannot express a
  small localized override without restating most of the inherited block.

## Method

- Live source review used current publisher/manual pages where they were
  directly reachable:
  [Biological Conservation guide](https://www.sciencedirect.com/journal/biological-conservation/publish/guide-for-authors),
  [Energy guide](https://www.sciencedirect.com/journal/energy/publish/guide-for-authors),
  [Springer Nature manuscript guidelines](https://www.springernature.com/gp/authors/publish-a-book/manuscript-guidelines),
  [Taylor & Francis CSE guide](https://www.tandf.co.uk/journals/authors/style/reference/tf_CSE.pdf),
  [Taylor & Francis NLM guide](https://files.taylorandfrancis.com/tf_NLM.pdf),
  [Chicago/Turabian notes and bibliography quick guide](https://www.chicagomanualofstyle.org/turabian/turabian-notes-and-bibliography-citation-quick-guide.html).
- Oracle verification used `node scripts/oracle-yaml.js styles/embedded/<style>.yaml --json`
  after repairing the broken `resolveStyleData` import in this PR.
- Parent/base comparison used direct Citum output diffs on the shared fixture:
  `cargo run --bin citum -- render refs -b tests/fixtures/references-expanded.json -c tests/fixtures/citations-expanded.json --mode both --show-keys`.
- The parent comparison counts below are changed output lines between the current
  style and the most plausible current parent candidate on that shared surface.
- Where this document says “current merge semantics,” it refers to the existing
  `extends:` contract: objects deep-merge, but arrays and explicit `null`
  values replace inherited content wholesale. That rule is simple and valid, but
  it means some bibliography/template deltas cannot be expressed as genuinely
  small wrappers today.

## Evidence Table

| Style | Guide / live relationship | Parent or template evidence | Current verification | Parent comparison | Classification | PR action |
| --- | --- | --- | --- | --- | --- | --- |
| `elsevier-harvard` | Biological Conservation still uses author-year reference instructions in its current guide. | CSL `rel: template` points to `ecology-letters`; no current Citum `ecology-letters` base exists. | `18/18` citations, `34/34` bibliography | `75` changed lines vs `apa-7th` | `thin profile on a new publisher/standards base` | Keep self-contained; do not remap to APA. Follow-up: `csl26-u1tq`. |
| `elsevier-vancouver` | Current Energy guide remains the journal authority; the embedded metadata still points to that guide. | Style identity is Elsevier NLM/Vancouver; there is no dedicated Elsevier numeric family base in Citum. | `18/18` citations, `34/34` bibliography | `106` changed lines vs `styles/nlm-citation-sequence.yaml` | `thin profile on a new publisher/standards base` | Keep self-contained; do not collapse directly to the repo NLM style. Follow-up: `csl26-u1tq`. |
| `springer-basic-author-date` | Springer Nature still documents a distinct “Basic style” and says it is based on Harvard style plus CBE recommendations. | Springer’s current guidance treats Basic as its own house style rather than as APA or Chicago. | `18/18` citations, `34/34` bibliography | `79` changed lines vs `apa-7th` | `thin profile on a new publisher/standards base` | Keep self-contained public style; dedicated Springer family root is follow-up work. Follow-up: `csl26-xt7k`. |
| `springer-basic-brackets` | Springer Nature still treats this as part of the same Basic house-style family. | CSL `rel: template` points to `springer-basic-author-date`. | `18/18` citations, `34/34` bibliography | `92` changed lines vs `springer-basic-author-date` | `thin profile on an existing base` | Do not land an `extends:` rewrite yet. Parentage is clear, but the current merge model only deep-merges objects; bibliography/type-variant arrays still replace wholesale. That makes the child delta too large to count as a meaningful thin wrapper, so the documented outcome is “keep taxonomy as profile; defer wrapper compression to follow-up design/work.” Follow-up: `csl26-xt7k`. |
| `springer-vancouver-brackets` | Springer Nature’s current guide says “Vancouver style” is based on NLM `Citing Medicine`. | House style is Springer-specific Vancouver, not generic IEEE or AMA. | `18/18` citations, `34/34` bibliography | `70` changed lines vs `styles/nlm-citation-sequence-brackets.yaml` | `thin profile on a new publisher/standards base` | Keep self-contained; it still needs a dedicated Springer/NLM family split rather than direct remap. Follow-up: `csl26-r4dm`. |
| `taylor-and-francis-council-of-science-editors-author-date` | Taylor & Francis currently publishes a dedicated CSE-9 author-date reference guide. | The guide names CSE directly; CSL `rel: template` points to `cse-name-year`. | `18/18` citations, `33/34` bibliography | `84` changed lines vs `styles/cse-name-year.yaml` | `thin profile on a new publisher/standards base` | Keep self-contained; parentage is real, but the missing standards/publisher base should be authored explicitly. Follow-up: `csl26-r4dm`. |
| `taylor-and-francis-national-library-of-medicine` | Taylor & Francis currently publishes a dedicated NLM brackets guide. | The guide and CSL metadata both point to NLM-with-brackets rather than AMA. | `18/18` citations, `34/34` bibliography | `36` changed lines vs `styles/nlm-citation-sequence-brackets.yaml` | `thin profile on a new publisher/standards base` | Keep self-contained; do not force it onto AMA or the current generic NLM style. Follow-up: `csl26-r4dm`. |
| `chicago-shortened-notes-bibliography` | Chicago/Turabian still defines notes-and-bibliography with shortened subsequent notes. | The current YAML already extends `chicago-notes-18th`; this remains the control case. | `34/34` citations, `32/33` bibliography | `61` changed lines vs `chicago-notes-18th` | `thin profile on an existing base` | No taxonomy change needed; keep as the proven control wrapper. |

## Decisions

- All eight candidate styles remain `profile` in registry taxonomy.
- No candidate was reclassified to `independent`.
- No new wrapper conversion was landed in this bean.
- `springer-basic-brackets` is the only strong existing-base relationship in the
  current repo, but the current merge model still forces a nearly full-file
  delta because bibliography/type-variant overrides are expressed through
  replace-whole structures rather than fine-grained inherited edits.
- The Elsevier, Springer-family-root, CSE, and NLM cases should be handled by
  explicit follow-up base-authoring work rather than by forcing these public
  styles onto today’s Tier-1 bases.

## Follow-Up Beans

- `csl26-u1tq` — author dedicated Elsevier family bases for Harvard and
  Vancouver public profiles.
- `csl26-r4dm` — author standards-backed CSE/NLM family bases for Taylor &
  Francis and Springer publisher profiles.
- `csl26-xt7k` — split Springer Basic family-root behavior from public wrappers
  and reduce `springer-basic-brackets` once the inherited bibliography delta is
  meaningfully smaller than the current file.
