---
# csl26-6eak
title: Tune gb-t-7714-2025-author-date to full fidelity
status: todo
type: task
priority: normal
tags:
    - style
    - fidelity
    - multilingual
created_at: 2026-07-16T10:56:59Z
updated_at: 2026-07-22T21:59:30Z
blocked_by:
    - csl26-8uxa
---

Drive the embedded gb-t-7714-2025-author-date style to 100% fidelity on the upstream corpus and flip its verification-policy benchmark run back to count_toward_fidelity: true with min_pass_rate 1.0. Citation-side behavior (无日期 / n.d. terms, eight ordered clusters) differs from numeric; re-run the cluster triage before tuning.

## Root cause found (2026-07-22): two precise, mechanical bugs, not scattered defects

Investigated the 0/203 raw baseline (119 ordering issues, 22 year-missing, 32
contributor mismatches from `node scripts/oracle.js
tests/fixtures/csl-m/gb-t-7714-2025-author-date.csl --json --scope bibliography
--refs-fixture tests/fixtures/test-items-library/gb-t-7714-2025.json`). All of
it traces to exactly two mechanical bugs in
`crates/citum-schema-style/embedded/styles/gb-t-7714-2025-author-date.yaml`'s
`bibliography.type-variants` block — not 119 independent defects.

### Mechanism

`gb-t-7714-2025-base.yaml`'s `bibliography.type-variants` is keyed by
comma-joined type-selector strings (e.g. `book,thesis,map`,
`chapter,entry-dictionary,entry-encyclopedia`, 14 keys total, numeric-shaped:
leading `number: citation-number` bracket, date positioned near
publisher/pages). `merge_bibliography_spec`
(`crates/citum-schema-style/src/style/overlay.rs:286`) merges a child style's
`type-variants` **per map key** into the resolved parent's map: "base keys not
in overlay are preserved" (see source comment). `TemplateVariants = IndexMap<
TypeSelector, TemplateVariant>` (order-preserving;
`crates/citum-schema-style/src/template.rs:77`), and
`resolve_type_variant`/`resolve_localized_type_variant`
(`crates/citum-engine/src/processor/rendering/grouped/component_predicates.rs`)
return the **first** entry (by iteration/insertion order) whose selector
matches the item's type.

Consequence: when author-date's key **string** matches base's exactly
(`IndexMap::insert` on an existing key updates in place, same position),
author-date's own override correctly replaces base's. When the strings
**differ** even by one alias, `insert` appends a **new** entry at the end —
base's original (numeric-shaped) entry, positioned earlier, still matches
first and wins. Author-date's own entry becomes silently dead code.

### Bug 1 — key-string mismatches shadow author-date's own overrides

Author-date's `bibliography.type-variants` has 10 keys; base has 14.
Comparing:

| Base key | Author-date key | Match? |
|---|---|---|
| `article,dataset,preprint` | `article,dataset` | **mismatch** (missing `preprint`) |
| `book,thesis,map` | `book,thesis,map,software` | **mismatch** (extra `software`) |
| `manuscript,personal_communication,pamphlet` | `manuscript,personal_communication` | **mismatch** (missing `pamphlet`) |
| `periodical` | — | **missing entirely** |
| `software` | — | **missing entirely** (only reachable via the mismatched book key above, which itself never wins) |
| `graphic` | — | **missing entirely** |
| (7 others: article-journal,article-magazine / article-newspaper / chapter,entry-dictionary,entry-encyclopedia / paper-conference / patent / webpage,post,post-weblog / report / standard) | exact match | OK — author-date's own template is used |

Every item whose type falls under a mismatched or missing key (book=93,
thesis=9, map=7, article=3, periodical/software/graphic, manuscript=3 —
well over half the 203-item corpus) silently renders through base's
numeric-shaped template: leading `[N]` citation-number bracket, date
positioned near publisher/pages instead of after the author. This is exactly
what the failing examples show, e.g. `gbt7714.5.1:1` (book):
`[1]博伯尔. 银行业的未来与人工智能[M]. 徐超，译. 北京：清华大学出版社，2023：35`
— numeric's bracket and date position, not author-date's.

**Fix:** rename author-date's mismatched keys to match base's exactly
(`article,dataset` → `article,dataset,preprint`; split
`book,thesis,map,software` into `book,thesis,map` + a separate `software` key
matching base's; `manuscript,personal_communication` →
`manuscript,personal_communication,pamphlet`), and author `periodical` and
`graphic` entries (currently absent).

### Bug 2 — the type-variants that DO fire are missing delimiters

For the 7 keys that already match (so author-date's own template *is* used),
the components have almost no `prefix`/`suffix`/`delimiter`/wrap punctuation
specified — unlike base's heavily-punctuated equivalents (colons after
container info, commas between contributors, periods between groups). Compare
`gbt7714.8.3.2:1` (chapter, uses author-date's own — matching — key):
oracle `阿扬，2023. 谈谈记忆：与诺贝尔获奖得者埃里克·坎德尔的问答[M]. 姜海伦，译//《环球科学》杂志社. 认识记忆力：关于学习、思考与遗忘的脑科学. 北京：机械工业出版社：15-18`
vs citum
`阿扬无日期谈谈记忆：与诺贝尔获奖得者埃里克·坎德尔的问答姜海伦《环球科学》杂志社认识记忆力：关于学习、思考与遗忘的脑科学北京机械工业出版社15-18`
— components run together with no separating punctuation at all.

Also noticed in passing: the `book,thesis,map,software` entry has `number:
edition` listed **twice** (once plain, once with `label-form: short`) —
likely an authoring/migration artifact, worth removing regardless of the
bigger fix.

**Fix:** add delimiters/prefixes/suffixes to each component, matching base's
punctuation conventions (colon-separated container/place groups,
comma-separated contributor lists, period between top-level segments),
adapted for author-date's front-loaded author+date ordering. Base's
`book,thesis,map` entry (`crates/citum-schema-style/embedded/styles/
gb-t-7714-2025-base.yaml`, top-level `bibliography.type-variants`, the
zh-CN/default-locale one — NOT the `bibliography.locales[0]` EN-scoped
partial override) is the closest reference for the punctuation pattern to
port, with the leading `number: citation-number` component dropped and
`date: issued, form: year` (with the same copyright/printing/accessed
`fallback:` chain used there, for GB/T §7.5.4.3 uncertain-date items) moved to
right after the `contributor: author` component instead.

The existing `message: term.no-date, form: short` component (already used
successfully in author-date's own **citation** section, e.g. lines 395,
416, 457, 509, 529) is confirmed as the correct building block for the
"render year, or 无日期/n.d. if absent" behavior — reuse it in bibliography
type-variants rather than a bare `date:` component, so both scopes share the
same no-date behavior.

### Scope estimate

~13 type-variant keys × ~10-15 components each need delimiter authoring/
verification against the corpus — a substantial but now fully mechanical
task with a clear, demonstrated pattern (no further architecture/engine
investigation needed). Deferred per 2026-07-22 session scoping decision
(best-effort this session; numeric + note landed at full 203-corpus adjusted
fidelity, author-date needs a dedicated follow-up pass). No CSL-M-oracle
"gold" reference exists for author-date structure the way
`data/GB-T_7714—2025.original.toml` (official standard text) did for
numeric's era/EDTF questions — the oracle's own structural shape (field
order, positioning) is the best available reference for this part, since it's
purely a reordering/punctuation question, not a content-correctness one like
the era-annotation case.

- [ ] Fix key-string mismatches (Bug 1): article,dataset,preprint /
      book,thesis,map + software / manuscript,personal_communication,pamphlet
      / add periodical, graphic keys
- [ ] Add delimiters/punctuation to all ~13 type-variant entries (Bug 2),
      porting base's punctuation pattern with author-date's field ordering
- [ ] Remove duplicate `number: edition` entry in book,thesis,map,software
- [ ] Re-verify against tests/fixtures/test-items-library/gb-t-7714-2025.json
      (target: adjusted 203/203, matching numeric/note)
- [ ] Flip verification-policy.yaml count_toward_fidelity: true, min_pass_rate: 1.0

## Progress (2026-07-22, later session): recipe validated to 68%, then reverted — new blockers found

Implemented the Bug 1 + Bug 2 fix plan above: rewrote `bibliography.type-variants`
to 14 keys matching base's exact selector strings (`article,dataset,preprint`;
`article-journal,article-magazine`; `article-newspaper`;
`chapter,entry-dictionary,entry-encyclopedia`; `periodical`;
`manuscript,personal_communication,pamphlet`; `book,thesis,map`; `software`;
`paper-conference`; `patent`; `graphic`; `webpage,post,post-weblog`; `report`;
`standard`), each built from a shared leading block:

```yaml
- group:
    - contributor: author
      form: long
      name-order: family-first
      delimiter: { mark: comma }
      sort-separator: ' '
      and: none
    - date: issued
      form: year
      fallback:
        - date: copyright
          form: year
          prefix: c
        - date: printing
          form: year
          suffix: 印刷
        - date: accessed
          form: year
          wrap: { punctuation: brackets }
        - message: term.no-date
          form: short
  delimiter: { mark: comma }
  suffix: '. '
```
followed by the rest of base's per-type fields (delimiters ported from base),
with body-position `date:issued form:year` occurrences removed (redundant
with the front block) but body-position `date:issued form:year-month-day`
occurrences **kept** — GB/T §7.5.4.2 mandates full-precision dates in the body
for online/report/patent/newspaper/archival types regardless of the
author-date front-matter year.

This surfaced a genuine **engine bug**, independent of this style: dates were
being silently deduplicated by `TemplateComponentTracker` (a
Contributor-only, `cs:substitute`-equivalent suppression mechanism) whenever
the same `date: issued` variable appeared twice in a template — which is
exactly what author-date's front+body dual-date shape does on purpose. Fixed
in `crates/citum-engine/src/processor/rendering/mod.rs`
(`TemplateComponent::Date(_) => None` in `get_variable_key`, full exemption)
and **landed separately** from this bean's YAML work, since it's a real
cross-cutting defect (also fixed a second, pre-existing occurrence in
`taylor-and-francis-chicago-author-date-core.yaml`'s `interview:` variant).
See commit history on `fix/gbt-date-annotation-fidelity` — do not re-derive
this fix if resuming this bean; it's already in `main`/merged upstream by the
time this bean is picked back up.

With the YAML recipe above **and** the engine fix, the corpus went from
**raw 0/203 → 111/203, adjusted 0/203 → 139/203 (68%)** — real, verified
progress. But **the YAML change was reverted** (not committed) before
closing out, because triaging the remaining 64 failures surfaced three new,
unresolved design questions the recipe above doesn't answer — landing it
would have meant shipping (and test-pinning) behavior with evidence against
its correctness:

### New finding 1 — no-author items: 佚名 (anonymous term), not title-substitution

`gbt7714.7.5.4.1:1` (Minguo-era, no author): oracle renders
`佚名，1947. [M]` (佚名 = "anonymous" — a placeholder **term**, à la
`term.no-date`'s "无日期"). The recipe above instead lets `contributor:
author` fall through to Citum's existing title-substitution behavior,
producing `1947（民国三十六年）. [M]` with no leading placeholder at all (and
in an earlier synthetic test, title substituting into the author slot
instead). **The real GB/T author-date convention needs an explicit
`message: term.anonymous`-shaped fallback in the author position**, mirroring
how `term.no-date` handles the missing-date case — this doesn't exist yet
and needs its own small feature, not just a punctuation port. Also directly
affects the archival "does the era annotation legitimately render twice
(front block + body block)" question raised by the reverted
`date_annotations.rs` assertions — unresolved, do not re-pin those tests
without re-deriving against a confirmed-correct rendering.

### New finding 2 — organizational author for `standard` type

`gbt7714.8.9.2:1`: oracle shows the issuing committee as the author slot
(`全国信息与文献标准化技术委员会，2021. GB/T 3792—2021 title[S]`); Citum's
`standard` type-variant has no path to promote the issuing-org/publisher
field into the author position, so it falls through to some other
substitution and drops the org name entirely. Likely needs a
`substitute.template` extension (currently only covers `editor`) — a
distinct, possibly non-trivial sub-feature, not addressed by the recipe.

### New finding 3 — disambiguation-suffix ordering swap

`gbt7714.9.3.1.3:2` / `:3`: two same-author-same-year references get `2000b`
/`2000c` swapped relative to the oracle's ordering. This looks like a
pre-existing disambiguation-ordering algorithm difference (not
type-variant/punctuation related) — plausibly out of scope for this bean
entirely; may deserve its own bean if confirmed.

### Where things stand

- The 14-key type-variant restructuring **shape** (base-matching keys, shared
  leading block, full-precision body dates preserved) is validated and
  should be the starting point for the next attempt — just don't reuse it
  verbatim for the author slot until finding 1 is resolved.
- Do not re-add the reverted YAML from `git log` blindly; it encodes the
  title-substitution behavior finding 1 says is wrong.
- Recommend resolving finding 1 (佚名 term) first — it's likely the single
  highest-leverage fix (affects the `7.5.4.1`/`7.5.4.3` cluster and probably
  more no-author items in the wider corpus), then finding 2 (org-as-author,
  narrower blast radius — `standard` type only), then assess finding 3
  separately.
