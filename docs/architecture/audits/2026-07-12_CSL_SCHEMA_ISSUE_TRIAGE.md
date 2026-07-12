# CSL Schema Open-Issue Triage

**Status:** Complete
**Version:** 1.0
**Date:** 2026-07-12
**Bean:** `csl26-rgd6`; follow-up work tracked under epic `csl26-kcda`
**Scope:** all 112 issues open on `citation-style-language/schema` at fetch time (2026-07-12)

## Purpose

This document triages every currently-open issue on the upstream
[`citation-style-language/schema`](https://github.com/citation-style-language/schema)
repository against Citum's own schema and engine, so that:

1. Citum maintainers know which upstream asks are already met and don't need
   revisiting, which are real gaps worth scheduling, and which don't apply.
2. The CSL project can see, issue by issue, which of its own open requests
   already have a working implementation to point to (Citum), independent of
   whether Citum ever becomes CSL-conformant in the traditional sense.

Every claim below is backed by a specific, checkable reference — a
JSON Schema path in [`docs/schemas/`](../../schemas/) (Citum's generated,
public data/style contract), a field in the canonical locale file
(`crates/citum-schema-style/embedded/locales/en-US.yaml`), or an entry in the
[Divergence Register](../adjudication/DIVERGENCE_REGISTER.md). Where a claim
could not be backed by a specific reference, the issue was placed in bucket 2
rather than asserted as resolved.

## Buckets

1. **Relevant to Citum, already addressed** — Citum implements the requested
   behavior, or has made and documented a deliberate decision not to (a
   written Divergence Register entry counts as addressed).
2. **Relevant to Citum, not addressed** — a real gap. Tracked as follow-up
   work (see below).
3. **Not relevant to Citum** — upstream repository process/governance/tooling,
   or CSL-XML/RELAX-NG mechanics that don't map onto Citum's declarative
   Rust/YAML model.

A note on bucket 2 specifically: several open issues propose their fix as a
*new CSL-XML template conditional or test attribute* (e.g. a new `<if>`
value-testing primitive). Citum's declarative model does not grow its
template language that way by design — new conditional primitives aren't
how Citum extends behavior. Where the underlying user need behind such an
issue is real, it's kept in bucket 2, but the evidence/notes column says so
explicitly and points at the kind of solution Citum would actually build
(a type/preset or a declarative style option) instead of implying Citum
should adopt the issue's literal proposed syntax.

## Result

| Bucket | Count |
|---|---|
| 1 — already addressed (40 direct + 13 partial) | 53 |
| 2 — real gap | 31 |
| 3 — not relevant | 28 |
| **Total** | **112** |

"Partial" bucket-1 entries are marked *(partial)* in the table: the core ask
is met, but with a caveat worth keeping (e.g. the mechanism covers most but
not all of the issue's stated cases).

## Scope and method

Issues were fetched via `gh issue list --repo citation-style-language/schema
--state open` (title, body, labels, URL; comment threads were read only where
the body alone was ambiguous). Each issue was checked against Citum's
generated JSON Schema (`docs/schemas/style.json`, `bib.json`, `citation.json`,
`locale.json`), the canonical `en-US.yaml` locale (terms, roles, grammar
options, messages, date/number formats), and the Divergence Register, rather
than against source code directly — the generated schema is Citum's public
contract and the artifact a CSL maintainer can most directly compare against
`csl-variables.rnc`/`csl-data.json`.

**Two known limitations of this pass**, disclosed rather than silently
absorbed into the buckets above:

- *Upstream currency* was not systematically checked — an issue may already
  be answered by text in the current CSL 1.0.2/1.1 spec without having been
  closed on GitHub. That would need a second pass against the spec itself,
  which this triage does not attempt beyond a few issues where it came up
  incidentally during evidence-gathering.
- *Engine-level* claims (as opposed to schema-level) were checked less
  rigorously than schema claims — several bucket-1/bucket-2 calls note this
  explicitly where the evidence is schema-only and the actual rendering
  behavior wasn't traced through `citum-engine`.

**This draft went through four correction rounds before being finalized**,
all driven by a domain-expert reviewer (a CSL project maintainer) catching
errors a schema-only scan produced:

- Round 1 (5 corrections): the initial pass under-built the reference surface
  — it read `terms:`/`vocab:`/`grammar-options:` from the locale file but
  missed the full `roles:` block, the `messages:` section, and the actual
  nested structure of `RawTermValue`. That produced false-gap calls on #63,
  #70, #80, #455, and #460 — see their *(partial)*/bucket-3 evidence notes,
  each marked "corrected on review."
- Round 2 (1 correction): a further hint to specifically re-check
  multilingual/locale and date machinery surfaced that #439's sortname ask is
  already met via `MultilingualComplex.sort-as` nested inside structured name
  parts — a mechanism not obvious from a first read of the schema.
- Round 3 (3 corrections): a category-level error — several issues whose
  *proposed solution* is a new CSL-XML template conditional (#62/#320's
  jurisdiction test, #436's `<if collection="parent">`, #377's
  `related=`/`original-variable=` test attribute) had been evaluated as if
  Citum might adopt that same conditional-language-growth pattern. Citum
  doesn't, by design. #377 moved from bucket 3 to bucket 1 (partial) —
  Citum's existing field-presence conditional already covers the underlying
  need via a different, pre-existing mechanism. #62/#320/#436 stayed in
  bucket 2 but with corrected notes describing the kind of fix Citum would
  actually build (a type/preset or a declarative option, not a new
  conditional) — the two affected follow-up beans were rewritten to match.
- Round 4 (4 corrections, from PR review comments, verified by dispatched
  code-reading agents rather than re-guessed): #147 was wrong the same way
  round 1's errors were wrong — evidence was read from a nested schema
  definition (`StyleBase`) without checking the wrapping type the `extends`
  field actually uses (`StyleReference`, which also resolves arbitrary
  parent styles by URI); corrected from "limited to built-in bases" to
  confirmed-broader. #455's "unconfirmed reachability" caveat (left open in
  round 1) is now resolved: confirmed reachable end-to-end via a
  `schemars(skip)`'d tolerant-enum fallback, which also surfaces that the
  *published* `bib.json` schema is stricter than actual runtime behavior.
  #379's "not confirmed configurable" was strengthened to "confirmed not
  configurable" against the actual punctuation-collision code. #410/#414/
  #438 (here, deposited) were reclassified from bucket 2 to bucket 1
  (partial) after direct source verification showed the relevant
  `GeneralTerm` enum variants and message IDs already exist — the real gap
  is unauthored locale *content*, not schema support, which is a materially
  smaller task than the original bucket-2 framing implied.

This is disclosed because it's material to how much independent weight to
give the "not addressed" calls that remain: they survived four rounds of a
maintainer checking specifically for errors, but that is not a guarantee
against a fifth.

## Bucket 1 — Relevant, already addressed

| Issue | Title | Evidence | Notes |
|---|---|---|---|
| [#13](https://github.com/citation-style-language/schema/issues/13) | Move publisher to cs:names | bib.json Publisher.anyOf includes $ref Contributor | Citum's Publisher type is anyOf[object{name,place}, Contributor, MultilingualString, string] — a publisher can already be a structured/personal-name-like Contributor, exactly what the issue asks |
| [#34](https://github.com/citation-style-language/schema/issues/34) | Define anonymous/et-al as names | DIVERGENCE_REGISTER div-010 (bibliography.options.anonymous-entries); SubsequentAuthorSubstituteRule, Substitute/SubstituteConfig in style.json | Citum has explicit substitute/anonymous-entries machinery for author-absent cases at render time; CSL-JSON-level representation question is resolved by Citum's own InputReference/Contributor model rather than left ambiguous |
| [#36](https://github.com/citation-style-language/schema/issues/36) | Multiple items per citation-number (chemistry compound) | DIVERGENCE_REGISTER div-001; bean csl26-zafv (issue #437 duplicate ask) | compound: true numeric-compound bibliography grouping implemented; same underlying ask as #437 |
| [#61](https://github.com/citation-style-language/schema/issues/61) | Align item types and field mappings (Zotero/Mendeley) | bib.json InputReference closed oneOf taxonomy (18 branches, MonographType/SerialType/etc) | Citum's redesigned closed reference-type taxonomy is a direct answer to type/field-mapping ambiguity across apps, by design, though it doesn't literally reconcile Zotero/Mendeley's own schemas |
| [#63](https://github.com/citation-style-language/schema/issues/63) | Locale-specific layouts (switch by item language) | bib.json MultilingualComplex/LangID/`language`/`field-languages` fields; style.json SortingMultilingualMode, SortingLocale | Corrected on review: Citum solves the underlying problem (foreign references should follow native-language formatting conventions) architecturally, by carrying language-tagged data through a locale-aware rendering/sorting pipeline, rather than requiring styles to duplicate an entire layout per source language the way the CSL-XML `<layout locale="...">` proposal does |
| [#94](https://github.com/citation-style-language/schema/issues/94) | Expand number of locator terms (plays, ancient sources, bibles, legal) | en-US.yaml terms: act, book, canon, chapter, column, line, scene, verse, article_locator all present | Every specific gap named in the 2011 issue (play act/scene/line, biblical book/chapter/verse, legal article) is already a locator term in Citum's canonical locale |
| [#109](https://github.com/citation-style-language/schema/issues/109) | Permit only one value on position attribute | citation.json Position is a single-valued field (not Vec<Position>) on CitationItem | Citum's schema is single-valued by construction — the tightening this issue asks for is structurally already true |
| [#112](https://github.com/citation-style-language/schema/issues/112) | Allow custom terms in style's cs:locale | crates/citum-schema-style/embedded/locales/overrides/en-US-chicago.yaml (per-style locale overrides exist) | Citum ships and supports per-style locale term overrides |
| [#113](https://github.com/citation-style-language/schema/issues/113) | Align CSL-JSON shortTitle vs schema title-short | bib.json variable union uses `short-title` consistently | Citum uses one consistent kebab-case field name; the naming inconsistency this issue reports doesn't exist in Citum's schema |
| [#130](https://github.com/citation-style-language/schema/issues/130) | Better JSON standardization (naming/type consistency) | bib.json uses consistent kebab-case field names throughout; NumOrStr $def applied uniformly to issue/volume/number-like fields | Both complaints in the issue (camelCase/kebab-case mixing, inconsistent string-vs-number typing) are resolved by construction in Citum's schema |
| [#147](https://github.com/citation-style-language/schema/issues/147) | Style inheritance and dependent styles | `extends:` is typed `Option<StyleReference>` (`crates/citum-schema-style/src/style/model.rs:56`); `StyleReference` = `Base(StyleBase) \| Uri(String)` (`style_base.rs:144-154`), and `docs/schemas/style.json:7135-7146` documents the URI form (`file://`, `https://`, `git+https://`, `cid:`) | Corrected on PR review: the first pass only read the nested `StyleBase` schema def and missed the wrapping `StyleReference` type the field actually uses. Arbitrary parent styles are resolved through a pluggable resolver chain (`crates/citum_store/src/resolver.rs`: file/HTTP/Git/CID/registry), confirmed by test (`style/tests.rs:380`, `extends: https://hub.citum.org/styles/apa-7th.yaml`) — this is a stronger match to the issue's ask than "dependent style aliasing," not a weaker one |
| [#160](https://github.com/citation-style-language/schema/issues/160) | CSL JSON literal/raw field exclusivity | bib.json `issued` etc. typed as plain EdtfString (no dual literal+structured object); SimpleName/StructuredName are distinct oneOf variants | Citum sidesteps the ambiguity by using EDTF strings for dates (no separate parsed/raw date-parts split) and distinct name variant types for names — mutual exclusivity is structural, not a validation rule bolted onto a shared object |
| [#162](https://github.com/citation-style-language/schema/issues/162) | Persian-language punctuation issues | en-US.yaml grammar-options: comma, semicolon, delimiters are per-locale configurable fields | Citum's locale model already carries per-locale delimiter/punctuation symbols (the exact class of fix this report needs), even though the report itself is a vague citeproc-js/Word-plugin bug |
| [#164](https://github.com/citation-style-language/schema/issues/164) | Document permitted values for circa field | bib.json EdtfString — dates use EDTF strings (which have a standard approximation/uncertainty marker) rather than a string\|number\|boolean circa field | Citum avoids the exact type-ambiguity this issue complains about by using EDTF instead of a separate untyped circa field |
| [#167](https://github.com/citation-style-language/schema/issues/167) | Expanded -short, original-, reviewed- variables | bib.json `original`/`reviewed`/`series` typed as WorkRelation (anyOf RefID \| full embedded InputReference) | A WorkRelation can embed a full related-work record (medium, container, pages, editor, etc.), which covers the "original-medium, original-pages, original-editor" style needs more generally than flat original-* variables would |
| [#240](https://github.com/citation-style-language/schema/issues/240) *(partial)* | Syntax to use terms in variable contents | en-US.yaml terms: `advance_online_publication` term exists; bib.json has a `status` field | The exact example cited (APA "Advance online publication") is already a named Citum term; the fully general "any variable can reference a term" syntax mechanism isn't separately confirmed |
| [#249](https://github.com/citation-style-language/schema/issues/249) | Support for journal special issues | en-US.yaml terms: `special_issue`, `special_section` both present | Direct hit — the exact labeling need in the cited APA/MLA/CMoS examples is a named Citum term |
| [#250](https://github.com/citation-style-language/schema/issues/250) *(partial)* | Adopt csl-m `alternative` (alt- prefixed variables) | bib.json `original` is WorkRelation (embeds full InputReference) | Reprint/original-work alternative info is covered via the `original` relation; no dedicated `translation` relation field confirmed, so the fully generic alt- mechanism csl-m proposes isn't a 1:1 match |
| [#278](https://github.com/citation-style-language/schema/issues/278) | Add CSL YAML | Citum's entire input/style authoring format is YAML, validated against generated JSON Schema (docs/schemas/*.json) | Citum is a working implementation of exactly this proposal — YAML input validated by a JSON-Schema-derived contract |
| [#316](https://github.com/citation-style-language/schema/issues/316) *(partial)* | Use YAML to maintain the input schema | Citum authors references/styles in YAML validated against JSON Schema generated from Rust source | Outcome (YAML-first authoring + JSON Schema validation) matches; the specific maintenance mechanism differs — Citum's schema source-of-truth is Rust types, not a YAML schema description |
| [#319](https://github.com/citation-style-language/schema/issues/319) *(partial)* | Convert input schema to object w/ metadata, add version property | bib.json root = {info: InputBibliographyInfo, references: [...], sets, custom}; InputBibliographyInfo = {title, author} | Object-with-metadata-and-references-array structure matches; no `version` property on the bibliography-file object specifically (InputBibliographyInfo lacks it) |
| [#321](https://github.com/citation-style-language/schema/issues/321) | Add abbreviation list mechanism | docs/schemas/abbrev-map.json exists as a dedicated generated schema | Direct hit |
| [#324](https://github.com/citation-style-language/schema/issues/324) | Preprocessing steps (separate data-wrangling from rendering) | citum-engine architecture: values/ layer resolves typed InputReference fields into intermediate render values before render/ stage (per crate outline) | Citum's pipeline already enforces exactly this separation as an architectural layer, not a per-processor convention |
| [#327](https://github.com/citation-style-language/schema/issues/327) | translated-title / transliterated-title on objectified title | bib.json Title = anyOf[string, StructuredTitle, MultilingualComplex, ...]; MultilingualComplex has original/lang/transliterations/translations | Direct hit — a title can carry both translated and transliterated forms via MultilingualComplex |
| [#332](https://github.com/citation-style-language/schema/issues/332) | Multilingual data and style structures | bib.json MultilingualString/MultilingualComplex/MultilingualName/LangID; used for Title, Publisher, contributor names | Field-level multilingual value/language/translation model matches the proposal's shape, applied broadly across the data model |
| [#334](https://github.com/citation-style-language/schema/issues/334) | Add fuller support for timestamps | bib.json all date fields (issued, accessed, created, available-date) are EdtfString | EDTF (the standard CSL 1.1 itself adopts) supports date-time values; Citum's date fields are EDTF strings, so timestamp precision is already representable |
| [#338](https://github.com/citation-style-language/schema/issues/338) | New delimiter names (cite/group/collapse delimiters) | style.json CitationGroupDelimiter, DelimiterPrecedesLast, DelimiterPunctuation, CitationCollapse — 4+ distinct named delimiter concepts | Citum already disambiguates delimiter concepts by separate named types rather than overloading one `delimiter` attribute |
| [#342](https://github.com/citation-style-language/schema/issues/342) | Style syntax for formatting multiple locators (e.g. volume:page) | style.json LocatorPattern: kinds+order+delimiter+label-repeat, "patterns tested in declaration order"; CitationLocator supports `segments: [LocatorSegment]` | Data model supports compound/segmented locators and style-level pattern-based formatting more generally than the issue's proposed single `collapse-volume-page-in-locator` flag |
| [#345](https://github.com/citation-style-language/schema/issues/345) | input: ensure consistency of dates, numbers | bib.json dates are EdtfString (single string, not array); locators are LocatorSegment objects | Citum's schema doesn't have the array-vs-object inconsistency this issue is worried about — dates are consistently EDTF strings, locators consistently structured objects |
| [#353](https://github.com/citation-style-language/schema/issues/353) | Legal support in CSL and CSLm (scope discussion) | bib.json has jurisdiction/reporter/docket-number/authority fields and legal_case/legislation terms directly in the core schema, no CSL-proper/CSL-M split | Citum already made the design decision this discussion is debating — generic legal-citation support lives directly in the unified schema |
| [#357](https://github.com/citation-style-language/schema/issues/357) | Style syntax for @related variables | style.json TemplateConditionField enum includes original-title, original-published, original-publisher, original-publisher-place | Concrete template-level syntax for referencing "original" related-work fields already exists as first-class conditionable/renderable fields |
| [#365](https://github.com/citation-style-language/schema/issues/365) | Additional citation modes (narrative citation without note) | citation.json CitationMode = {integral, non-integral}; integral = "Author inline in text... Also known as narrative or in-text citations" | Direct hit for the narrative/in-text citation mode described |
| [#369](https://github.com/citation-style-language/schema/issues/369) | Add "in-parens" / "in-brackets" formatting attributes | style.json WrapPunctuation enum = {parentheses, brackets, quotes}; WrapConfig/LabelWrap/SegmentWrap/BibliographyLabelWrap | Direct hit — semantic wrap-in-parens/brackets/quotes already exists as formatting attributes, not just literal affixes |
| [#371](https://github.com/citation-style-language/schema/issues/371) | Special term="type" handling | en-US.yaml terms keyed directly by reference type name (thesis, webpage, report, article_journal, etc.) | Citum's terms are already keyed by type name, so "the term for the item's type" is a direct lookup — no special term="type" magic value needed |
| [#377](https://github.com/citation-style-language/schema/issues/377) *(partial)* | Add tests for related variables | style.json TemplateConditionField already includes original-title, original-published, original-publisher, original-publisher-place (same evidence as #357) | Corrected: misread on first pass — this is not upstream test-suite/CI housekeeping, it's a proposal for new `<if>` conditional syntax (`related="original"` or `original-variable="title"` attributes) to test related/original variables. Citum won't adopt that specific template-language extension, but the underlying capability — testing whether an original-related field is present — already exists via Citum's own (differently-shaped, pre-existing) field-presence conditional system. Same underlying need as #357, addressed the same way |
| [#379](https://github.com/citation-style-language/schema/issues/379) *(partial)* | Make punctuation collapsing localisable | en-US.yaml grammar-options (punctuation-in-quote, nbsp-before-colon, serial-comma, etc.) are per-locale fields; punctuation-collision resolution itself is a hardcoded match table (`citum-engine/src/render/citation.rs:15-55` `resolve_punctuation_collision`; `bibliography.rs:373-391` `DANGLING_PUNCTUATION_PATTERNS`) with no locale hook | General locale-configurable punctuation options exist, but strengthened on PR review: the specific ask (suppress comma after titles ending in !/?) is confirmed **not** independently configurable — grammar-options only wires quote/comma characters into rendering, never the collision table itself. Current default doesn't even suppress in this case (`('!', ',') => "!,"` keeps both marks) |
| [#387](https://github.com/citation-style-language/schema/issues/387) | Archival copies of online content (web.archive.org) | bib.json ArchiveInfo, `archive`, `archive-location` fields | Direct hit |
| [#388](https://github.com/citation-style-language/schema/issues/388) *(partial)* | Add a "primary source" variable | bib.json `original` WorkRelation (embeds full InputReference or RefID); en-US.yaml `classic` is a term only, not a MonographType/genre value | The original-work relation covers citing primary source + edition-of-access as two linked records; correcting an initial read — `classic` exists only as a label term, there is no backing `classic` reference type/genre, so that half of the issue's cited prior art doesn't actually exist in Citum yet |
| [#410](https://github.com/citation-style-language/schema/issues/410) *(partial)* | New term "of" | `GeneralTerm::Of` already exists as a wired enum/message-ID (`crates/citum-schema-style/src/locale/types.rs:195`) | Reclassified on PR review from bucket 2: the type layer already supports this term; only the actual term text in `en-US.yaml` is unauthored — a one-line locale-content addition, not a schema gap |
| [#411](https://github.com/citation-style-language/schema/issues/411) | `assignee` variable for patents | bib.json `assignee` field present on patent-shaped InputReference branch | Direct hit |
| [#414](https://github.com/citation-style-language/schema/issues/414) *(partial)* | Add a term "to" | `GeneralTerm::To` already exists as a wired enum/message-ID (`locale/types.rs:197`) | Reclassified on PR review from bucket 2, same reasoning as #410 |
| [#437](https://github.com/citation-style-language/schema/issues/437) | Support for numeric compound styles | DIVERGENCE_REGISTER div-001; bean csl26-zafv | Duplicate ask of #36, already completed |
| [#438](https://github.com/citation-style-language/schema/issues/438) *(partial)* | Terms and variables 1.0.3 (here, deposited, translated-title) | `GeneralTerm::Here`/`::Deposited` already exist as wired enum/message-IDs (`locale/types.rs:221,223`); translated-title already covered (see #327) | Reclassified on PR review: all three items are now schema/type-addressed. `here`/`deposited` still need `en-US.yaml` content (locale-authoring, not schema work); `deposited` should also get a `pattern.deposited-date` composition alongside the flat term, matching the `accessed`/`retrieved` precedent |
| [#439](https://github.com/citation-style-language/schema/issues/439) *(partial)* | Sorting Fields (sortname/sorttitle/presort overrides, empty-author-last) | bib.json StructuredName.given/family are typed MultilingualString = anyOf[string, MultilingualComplex], and MultilingualComplex has a `sort-as` override field | Corrected on review: `sort-as` isn't limited to titles — since given/family name parts are MultilingualString too, a `sortname`-equivalent override is available at the individual name-part level, more granular than biblatex's single sortname field. Kept as partial: "sort empty author names at the end instead of the beginning" specifically isn't confirmed configurable |
| [#441](https://github.com/citation-style-language/schema/issues/441) | Request for Manual Type | bib.json MonographType enum includes `manual` | Direct hit |
| [#444](https://github.com/citation-style-language/schema/issues/444) *(partial)* | Periodical with columns instead of pages | en-US.yaml `column` locator term exists (singular/plural/short); bib.json `page`/`pages` fields are generic NumOrStr-based, not page-specific in type | The in-text locator term for citing a column already exists; the reference's own extent field is generically typed so it can technically hold a column range, just without column-specific semantic labeling |
| [#450](https://github.com/citation-style-language/schema/issues/450) *(partial)* | 'and' <> '&' in terms (style-driven and-form selection for compound role terms) | en-US.yaml `and` term has both long ("and") and symbol ("&") forms | The long/symbol duality mechanism exists at the term level; whether the editor-translator compound role label automatically follows the style's chosen and-form isn't independently confirmed |
| [#451](https://github.com/citation-style-language/schema/issues/451) | Canonical reference locator (combined book/chapter/line/etc.) | style.json LocatorPattern (kinds+order+delimiter+label-repeat) + CitationLocator.segments — same mechanism confirmed for #342 | Compound/segmented locators with configurable per-pattern rendering directly support the canonical-reference-locator use case described |
| [#452](https://github.com/citation-style-language/schema/issues/452) | More flexible disambiguation | DIVERGENCE_REGISTER div-009 (year-suffix keyed on issued year only, not full rendered date string); bean csl26-xrc5 / docs/specs/DISAMBIGUATION.md (short-title + first-reference-note-number) | Both concrete complaints in this issue are addressed: div-009 fixes exactly the citeproc-js full-string-keying failure mode described, and csl26-xrc5 implements the short-title/first-reference-note-number combination requested |
| [#453](https://github.com/citation-style-language/schema/issues/453) | Syntax for controlled vocabularies | en-US.yaml `vocab: genre/medium` maps (canonical key -> localized term), locale.json RawVocab | Citum's vocab mechanism is a native, type-level implementation of controlled-vocabulary localization — cleaner than the string-prefix (`term:working-paper`) convention the issue proposes, same outcome |
| [#455](https://github.com/citation-style-language/schema/issues/455) | textual-editor (editor vs editorial-director role split) | style.json ContributorRole (22 values) includes both `editorial-director`/`textual-editor`; data-side `ContributorRole` (`crates/citum-schema-data/src/reference/contributor.rs:197`) uses a `tolerant_enum!` macro with an `Unknown(String)` catch-all (`macros.rs:36,76-79`); `contributor_role_to_reference_role` (`citum-engine/src/values/contributor/mod.rs:72-77`) maps style-side EditorialDirector/TextualEditor to the matching Unknown string | Corrected on PR review, now confirmed end-to-end rather than left open: `role: editorial-director` in input data deserializes without error (no schema-strict validation gate in the citum-io input path) and renders the correct locale label. FYI for readers: the generated `docs/schemas/bib.json` is stricter than actual runtime — the `Unknown` catch-all is `schemars(skip)`, so the published schema doesn't show that unrecognized role strings are accepted |
| [#460](https://github.com/citation-style-language/schema/issues/460) | Support gender attribute on single/multiple | locale.json RawTermValue's singular/plural variant: `singular` and `plural` are each independently typed as RawGenderedString | Corrected on review: I only checked the term-wide `gender` sibling field on RawLocatorTerm and missed that RawTermValue's singular/plural forms are independently gendered; the schema already supports gender varying by number, en-US.yaml just doesn't populate a case that needs it yet (data-authoring gap in one locale, not a schema gap) |
| [#463](https://github.com/citation-style-language/schema/issues/463) | License field missing from json schema | bib.json `license` field present (SPDX identifier) on relevant InputReference branches | Citum already has the field the upstream issue reports missing from its own csl-data.json |

## Bucket 2 — Relevant, not addressed

Tracked as follow-up work under epic `csl26-kcda`, grouped into 16 themed
beans covering the 31 bucket-2 issues (issues are grouped by Citum-side
feature area, not 1:1 with beans — five of the sixteen are one issue each,
per review feedback that some rendering/formatting gaps shouldn't be
bundled together). One additional small bean (`csl26-rnrd`) tracks a
bucket-1 (partial) follow-up rather than a bucket-2 gap; included in the
table below for completeness.

| Bean | Theme | Issues |
|---|---|---|
| `csl26-6ne6` | Additional locator terms (legal/liturgical/regional) | #343, #346, #412, #418 |
| `csl26-ulpk` | Math locators | #440 |
| `csl26-21d9` | Configurable title-casing / sort-key stop words | #106, #454, #456 |
| `csl26-s2zn` | Legal citation gaps (jurisdiction-aware rendering, ECLI, statute amendments) | #62, #131, #320, #339, #350 |
| `csl26-013w` | Contributor role & name gaps | #361, #424 |
| `csl26-fqug` | New reference types (indigenous knowledge, sacred works) | #446, #447 |
| `csl26-d5vu` | Small term additions (schema-level gaps only — see note) | #443, #445 |
| `csl26-kzg8` | Container/chapter bibliography collapsing | #370, #436 |
| `csl26-v3g2` | Data-model additions (social handle, multiple URLs) | #432, #462 |
| `csl26-eyit` | Locale date-grammar refinement (verb forms, era-term position) | #458, #459 |
| `csl26-cbcp` | Name-sort-order: all-but-last-inverted option | #134 |
| `csl26-p03v` | Page-range-format trust policy (no "expanding" ambiguous input) | #81 |
| `csl26-cl4q` | Open-ended page-range support (e.g. "12ff") | #372 |
| `csl26-i41r` | Bibliography entry-separator option | #386 |
| `csl26-5fyz` | Strip-protocol URL rendering option | #395 |
| `csl26-7ip9` | Cross-role names-collapse mechanism | #442 |
| `csl26-rnrd` | `version` property on bibliography-file metadata | #319 (bucket 1, partial — small follow-up) |

`csl26-d5vu`'s issue list narrowed on PR review: #410/#414/#438 (here,
deposited) moved to bucket 1 (partial) above — `GeneralTerm::Of`/`::To`/
`::Here`/`::Deposited` already exist at the type level, so what remains is
`en-US.yaml` content authoring, not schema design. `csl26-pq40` (the
bundled "rendering/formatting gaps" bean) was deleted and split into five
standalone beans, one per issue, per review feedback — the reviewer wasn't
willing to commit to addressing all five original issues as one unit.

| Issue | Title | Evidence | Notes |
|---|---|---|---|
| [#62](https://github.com/citation-style-language/schema/issues/62) | Testing for URN:LEX jurisdiction values | bib.json has `jurisdiction` field; TemplateGroupCondition is field-present/field-absent only, not value comparison | Corrected framing: the proposal (and its duplicate #320) asks for a new value-testing conditional in the template language, which Citum's declarative model doesn't grow by design — Citum's existing conditional mechanism (TemplateConditionField) only tests field presence/absence, not "does field X equal Y", so adding jurisdiction to that list wouldn't even satisfy the ask as stated. The underlying need (render legal citations differently by jurisdiction) is real, but Citum would more likely solve it via a jurisdiction-aware type/preset (like MonographType/CollectionType) than by extending conditional syntax. Groups with #320. |
| [#81](https://github.com/citation-style-language/schema/issues/81) | Do not mangle strings on page-range-format=expanded | no found design doc on page-range expansion trust/mangling policy | No DIVERGENCE_REGISTER entry or spec found addressing this; needs verification against renderer PageRangeFormat behavior |
| [#106](https://github.com/citation-style-language/schema/issues/106) | Word lists for title casing and sorting (stop words) | no configurable stop-word/sort-word-list found in style.json sort surface | No evidence of a configurable stop-word mechanism; groups with #454 (sort while dropping articles) |
| [#131](https://github.com/citation-style-language/schema/issues/131) | European Case Law Identifier (ECLI) | bib.json has `standard-number`, `docket-number`, `authority`, `jurisdiction` but no ECLI-specific field | No dedicated field; standard-number could serve as a generic container but isn't ECLI-specific or validated |
| [#134](https://github.com/citation-style-language/schema/issues/134) | Extend name-as-sort-order with all-but-last option | style.json NameSortOrder enum = {family-given, given-family} only | No all-but-last/all-except-last variant present |
| [#320](https://github.com/citation-style-language/schema/issues/320) | New condition: test for jurisdiction | style.json TemplateConditionField is a closed field-presence/absence list (17 values), not a value-comparison mechanism; jurisdiction not among them either way | Corrected framing: this is a request for a new CSL-XML-style value-testing conditional (`<if jurisdiction="EU">`), a category of template-language growth Citum's declarative model doesn't take on by design. The underlying need (jurisdiction-aware legal-citation rendering) is real and unaddressed, but the fix Citum would build is a jurisdiction-aware type/preset, not a new conditional test — see revised bean csl26-s2zn. Groups with #62 (older duplicate stub) |
| [#339](https://github.com/citation-style-language/schema/issues/339) | Add variables for statute-amendment versioning | no amendment-date/in-force-date/gazette-citation field found in bib.json variable union | Genuine gap for this legal-citation-specific need |
| [#343](https://github.com/citation-style-language/schema/issues/343) | Locator for "surah" (Quran divisions) | `surah` absent from en-US.yaml terms | Genuine small gap; groups with #412 (recital), #418 (Bande/Jahrgang), #440 (math locators) as "additional locator terms" |
| [#346](https://github.com/citation-style-language/schema/issues/346) | Additional locators (AGLC: clause, division, schedule, sub-clause, subdivision, sub-paragraph, subsection) | none of these 7 terms present in en-US.yaml terms | Genuine gap; groups with #343/#412/#418/#440 as "additional locator terms" theme |
| [#350](https://github.com/citation-style-language/schema/issues/350) | `identifiers` object variable | bib.json has named fields (doi, isbn, issn, standard-number, docket-number, patent-number, ads-bibcode) but no generic identifiers array/map and no ECLI/ISMC/ISWC fields | Same underlying gap as #131 (ECLI) — no home for identifier types outside the named-field list, and no extensible identifiers container |
| [#361](https://github.com/citation-style-language/schema/issues/361) | Specify creator sub-role in names data | bib.json ContributorEntry = {role: ContributorRole (15-value closed enum), contributor, gender} — no free-text/custom sub-role field | No mechanism for a custom per-contributor role label (e.g. "cartographer") beyond the closed role enum |
| [#370](https://github.com/citation-style-language/schema/issues/370) | Shorten references to containers if multiple chapters cited | no general "shorten repeated container across entries" mechanism found; DIVERGENCE_REGISTER div-010 covers a related but distinct case (anonymous-entries container-led reordering) | Genuine gap for the general multi-chapter-same-container shortening behavior |
| [#372](https://github.com/citation-style-language/schema/issues/372) | Open-ended page ranges (e.g. "12ff") | style.json PageRangeFormat enum = {expanded, minimal, minimal-two, chicago, chicago16} — no open-ended variant | Genuine gap |
| [#386](https://github.com/citation-style-language/schema/issues/386) | Suppress newline between bibliography entries | no entry-separator/newline-suppression option found in style.json bibliography surface | No evidence of this render-layer option; needs confirming against citum-engine's per-output-format bibliography renderers before treating as settled |
| [#395](https://github.com/citation-style-language/schema/issues/395) | Remove protocol from URL | style.json LinksConfig = {doi, url, target: LinkTarget, anchor: LinkAnchor} — no protocol-stripping option found | No evidence of a strip-protocol/unprefixed-URL rendering option |
| [#412](https://github.com/citation-style-language/schema/issues/412) | New locator term `recital` (EU legal) | `recital` absent from en-US.yaml terms | Groups with #343/#346/#418/#440 as "additional locator terms" theme |
| [#418](https://github.com/citation-style-language/schema/issues/418) | New locator term to distinguish "Bande"/"Jahrgang" (volume-book vs volume-periodical) | only a single generic `volume` term present, no type-dependent variant | Genuine gap, though Citum's terms-keyed-by-type architecture (seen in #371's evidence) would readily accommodate a type-dependent term variant if added |
| [#424](https://github.com/citation-style-language/schema/issues/424) | Name particle abbreviator (e.g. "von" -> "v.") | bib.json StructuredName has dropping-particle/non-dropping-particle/family/given/suffix (particle storage, same as CSL-JSON always had); no configurable particle-abbreviation rendering rule found | Data storage for particles isn't new; the specific automatic-abbreviation-on-render feature this issue asks for isn't confirmed present |
| [#432](https://github.com/citation-style-language/schema/issues/432) | Handle/username for tweets, Instagram, TikTok | bib.json `platform` = software platform (Windows/macOS/Linux); `network` = broadcaster/streaming network — neither is a social-media handle field; no handle/username field found anywhere in InputReference | Genuine gap — initial scan of the variable union was misleading; targeted check of field descriptions confirms platform/network don't cover this |
| [#436](https://github.com/citation-style-language/schema/issues/436) | Collapse multiple child items into one parent item (shared container entry) | bib.json `container` field is a WorkRelation (structural parent/child link) already exists; DIVERGENCE_REGISTER div-010 (anonymous-entries: container-led) is a precedent for consolidating entries by container, but scoped to the anonymous-author case | Corrected framing: the issue's proposed solution is a brand-new CSL-XML conditional (`<if collection="parent">`/`<else-if collection="child">`) plus new elements/terms — template-language growth Citum's declarative model won't take on. The underlying need (one shared bibliography entry for a multi-author collected work, individually-citable chapters, footnote cross-reference "see N.1") is real and unaddressed, but Citum already has the relevant building blocks (the `container` relation, and div-010's container-led option as a precedent for how Citum would extend this declaratively — a style-level bibliography option, not a template conditional). Same underlying gap as #370; groups together as "container-collapsing" theme |
| [#440](https://github.com/citation-style-language/schema/issues/440) | Need Math Locators for CSL (theorem, lemma, algorithm, problem, definition, proposition, corollary) | none of these 7 terms present in en-US.yaml | Clear, cohesive gap — own theme ("math locators") rather than folded into general locator-term gap |
| [#442](https://github.com/citation-style-language/schema/issues/442) | [CSL 1.1] Collapse <names> variables (cross-role name-list collapsing) | style.json CitationCollapse enum = {citation-number} only; no cross-role names-collapse mechanism found | Genuine gap |
| [#443](https://github.com/citation-style-language/schema/issues/443) | Add "idem" for identical author and editor | `idem`/"id." absent from en-US.yaml terms; loc_cit/op_cit exist but not this term | Small gap, conceptually adjacent to existing ibid/loc_cit infrastructure and SubsequentAuthorSubstituteRule |
| [#445](https://github.com/citation-style-language/schema/issues/445) | Missing labels for number variables (part-number, supplement-number) | en-US.yaml has `part`/`supplement` as locator terms, but no `part_number`/`supplement_number` entries in the number-variable-label term group (unlike chapter_number, collection_number, number_of_pages, etc. which do exist) | Verified, real, and shared with upstream — Citum has the same specific gap being reported here |
| [#446](https://github.com/citation-style-language/schema/issues/446) | Indigenous sources of knowledge (new reference type) | no matching MonographType/CollectionType/genre value found | Genuine gap; not a mechanical term addition — needs its own citation-form design work, flagged as a distinct theme rather than folded into generic type-taxonomy gaps |
| [#447](https://github.com/citation-style-language/schema/issues/447) | Sacred work type (Bible/Koran, no punctuation, no bibliography entry) | no `sacred-work` MonographType/genre value; `classic` is a term only, not a backing type (see corrected #388 note) | Genuine gap |
| [#454](https://github.com/citation-style-language/schema/issues/454) | Sort while dropping articles (stop words) | no configurable stop-word/sort-word-list found | Same gap as #106; groups together |
| [#456](https://github.com/citation-style-language/schema/issues/456) | Follow updated Chicago title casing rules (5+-letter preposition capitalization) | no configurable title-casing stop-word list found (same absence as #106/#454) | Chicago-18th-specific manifestation of the general title-casing/stop-word gap; groups with #106/#454 |
| [#458](https://github.com/citation-style-language/schema/issues/458) | Verb forms for dates (locative vs nominative grammatical case) | en-US.yaml roles have verb/verb-short forms; RawDateTerms has no equivalent verb-form field found | Verb-form infrastructure exists for role terms but not confirmed for date-related grammatical case variation |
| [#459](https://github.com/citation-style-language/schema/issues/459) | Position parameter for era terms (AD/BC placement before/after year) | en-US.yaml ad/bc/bce/ce are plain terms with no position/placement field | Genuine gap |
| [#462](https://github.com/citation-style-language/schema/issues/462) | Multiple URLs for the same item | bib.json `url` field type = ["string","null"] (single value, not array) | Confirmed single-valued; genuine gap |

## Bucket 3 — Not relevant to Citum

| Issue | Title | Reason |
|---|---|---|
| [#54](https://github.com/citation-style-language/schema/issues/54) | strip-periods on cs:text macro/cs:name-part/cs:et-al | Citum has no cs:text/macro indirection; this is a rule about where a CSL 1.0 XML attribute may legally appear, not a data/behavior gap |
| [#68](https://github.com/citation-style-language/schema/issues/68) | Suppress Ibid when first on page | Requires physical page-break awareness from the typesetting/document layer; out of scope for a citation processor engine (Citum included) |
| [#70](https://github.com/citation-style-language/schema/issues/70) | id should be optional in Embedded Citation Object Format | Corrected on review: the issue concerns the legacy "Embedded Citation Object Format" used by reference-manager Word/LibreOffice plugins to round-trip citation state inside a document, a different artifact from Citum's engine-input CitationItem type; Citum doesn't implement or need that document-embedding format, and CitationItem.id being required is fundamental to resolving which reference is cited, not an analogous optionality question |
| [#80](https://github.com/citation-style-language/schema/issues/80) | Test condition context=citation/bibliography | Corrected on review: this issue exists to work around CSL-XML styles being forced to share one macro hierarchy between cs:citation and cs:bibliography; Citum's declarative model gives citation and bibliography their own separate template configuration rather than one conditionally-branching shared tree, so the context-bleed problem this conditional is meant to solve doesn't arise |
| [#108](https://github.com/citation-style-language/schema/issues/108) | type as implicitly match=any | Citum's type-variants resolve per-type directly rather than via boolean match logic on type lists; the ambiguity this issue fixes doesn't exist in Citum's model |
| [#118](https://github.com/citation-style-language/schema/issues/118) | Add description fields into JSON schema | About annotating citation-style-language/schema's own artifact; not a Citum gap (Citum's generated docs/schemas/*.json already carry description fields throughout, for reference) |
| [#126](https://github.com/citation-style-language/schema/issues/126) | Relicense schema under MIT license |  |
| [#135](https://github.com/citation-style-language/schema/issues/135) | Disable RELAX NG whitespace normalization | Citum has no RNC schema; not applicable |
| [#190](https://github.com/citation-style-language/schema/issues/190) | Add title/description annotations to JSON Schema | Duplicate of #118's ask, upstream-only |
| [#195](https://github.com/citation-style-language/schema/issues/195) | Update styles & locales to CC BY-SA 4.0 |  |
| [#220](https://github.com/citation-style-language/schema/issues/220) | add CHANGELOG.md file |  |
| [#223](https://github.com/citation-style-language/schema/issues/223) | Script to update json schema strings from one file |  |
| [#238](https://github.com/citation-style-language/schema/issues/238) | Define branch/tag versioning strategy for repo(s) |  |
| [#244](https://github.com/citation-style-language/schema/issues/244) | Create a MakeCSL tool | citum-migrate converts existing CSL 1.0 XML to Citum YAML, which is a different problem (conversion, not generation from formatted-citation examples) |
| [#267](https://github.com/citation-style-language/schema/issues/267) | refactor rnc schema | Upstream-only, no RNC in Citum |
| [#275](https://github.com/citation-style-language/schema/issues/275) | CI,tools: Add validation step on rnc schemas |  |
| [#279](https://github.com/citation-style-language/schema/issues/279) | Add JSON schemas to JSON store | Not a schema-content gap |
| [#347](https://github.com/citation-style-language/schema/issues/347) | input: move noteIndex property | Upstream schema-authoring bug report, not a Citum data-model question |
| [#349](https://github.com/citation-style-language/schema/issues/349) | input: uris, mendeley properties | Citum's InputReference is a clean redesign with a generic `custom` field for extensions rather than carrying forward app-specific legacy properties |
| [#352](https://github.com/citation-style-language/schema/issues/352) | Tests for cs:intext configurations |  |
| [#354](https://github.com/citation-style-language/schema/issues/354) | Create 1.0 -> 1.1 style upgrade script | Different problem from citum-migrate (CSL-1.0-XML-to-Citum-YAML), not upstream's ask |
| [#406](https://github.com/citation-style-language/schema/issues/406) | Planning 1.1.0 release |  |
| [#407](https://github.com/citation-style-language/schema/issues/407) | Differences between csl-variables.rnc and csl-data.json |  |
| [#408](https://github.com/citation-style-language/schema/issues/408) | Update readme.md |  |
| [#417](https://github.com/citation-style-language/schema/issues/417) | Disable cs:text attributes for macros (precedence rules) | No macro/cs:text indirection in Citum's template model |
| [#426](https://github.com/citation-style-language/schema/issues/426) | Questions about variables "categories" and "printing" | Not a Citum data-model question |
| [#429](https://github.com/citation-style-language/schema/issues/429) | git: 1.1 conflicts, merge commits, message guidelines |  |
| [#433](https://github.com/citation-style-language/schema/issues/433) | Render numbered bibliography entry without a tab | Citum's bibliography renderers are independently designed per output format; not confirmed to have this specific artifact, and not a schema/spec question either way |
