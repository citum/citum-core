# Embedded Core Styles — Guide-Conformance Audit

**Date:** 2026-06-20
**Status:** First sweep complete
**Related:** `.beans/csl26-53zy--guide-conformance-review-of-embedded-core-styles.md`

All 15 embedded core styles reviewed against the citeproc reference (and the
published guide where reachable). Clean, low-risk YAML fixes landed per style;
structural rewrites and engine-level residuals are documented and deferred.
**IEEE, APA 7, Chicago author-date, AMA 11, MLA 9, Elsevier Vancouver, and
Springer ×3** received fixes; **Chicago notes** and the **T&F trio** are
flagged as deeper work (the latter needs the sandboxed T&F PDFs).

## Why this audit

Expanded test fixtures surfaced rendered output (notably IEEE) that looked wrong
against the actual published style guides. This audit reviews each **embedded
core style** (`crates/citum-schema-style/embedded/styles/`, symlinked as
`styles/embedded/`) two ways:

1. **Metadata** — does `info.source` point at the correct, most-authoritative
   guide, and does the version label match the guide's current revision?
2. **Conformance** — does the Citum YAML render each reference type the way the
   guide prescribes?

### Method

Three-way comparison per style, to locate *where* a divergence originates:

- **Citum** — `citum render refs -s styles/embedded/<name>.yaml -b tests/fixtures/references-expanded.json`
- **citeproc-js reference** — the oracle snapshot `tests/snapshots/csl/<csl-id>.json`
- **Published guide** — the source document named in `info.source.links`.

### Finding buckets

- **(A)** Citum YAML wrong vs guide — fixable in the style.
- **(B)** citeproc/CSL reference wrong vs guide — upstream; note, decide whether
  to diverge intentionally.
- **(C)** fixture data mistyped — note for a fixture follow-up; out of scope here.
- **(D)** guide inaccessible / rule genuinely ambiguous — flagged, not guessed.
- **(E)** engine bug — affects every style sharing the code path; fix in engine.

> **Reframing the original hypothesis.** The worry was that the citeproc
> "expected output" was wrong. For IEEE the opposite is mostly true: the
> citeproc reference largely matches the guide, and the divergences were in the
> **Citum YAML** and one **engine** path. The citeproc snapshots are a reliable
> proxy for CSL-derived guides on common types.

---

## IEEE — `ieee.yaml`

Guide: **IEEE Reference Guide, v3.28.2025** (© 2025 IEEE), supplied as the
primary source and cross-checked against the IEEE Author Center pages and the
Murdoch University reproduction of the official examples.

### Metadata findings

| Field | Current | Assessment |
|---|---|---|
| `info.title` | "IEEE Reference Guide version 11.29.2023" | **Stale (D/metadata).** Current guide is **v3.28.2025**. Title embeds a version that is two revisions behind. |
| `rel: documentation` | `journals.ieeeauthorcenter.ieee.org/.../ieee-editorial-style-manual/` | **Gateway, not the guide (verified via fetch).** That page links *out* to the IEEE Reference Guide; it contains no reference-format examples. The authoritative doc is the *IEEE Reference Guide* (now served as a Google Doc; the historical `ieeeauthorcenter.ieee.org/wp-content/uploads/IEEE-Reference-Guide.pdf` 301-redirects to it). |
| `csl-id` / `rel: self` | `zotero.org/styles/ieee` | OK. |

### Conformance — per type (Citum vs guide)

Guide reference templates (verbatim, abbreviated):
- **Journal:** `J. K. Author, "Name of paper," Abbrev. Title of Periodical, vol. x, no. x, pp. xxx–xxx, Abbrev. Month, year.` — article title in quotes, **comma inside** the closing quote, periodical italic, **entry ends with a period** (except URL-terminated).
- **Book chapter:** `J. K. Author, "Title of chapter," in Title of Book, X. Editor, Ed., City, State, Country: Publisher, year, pp. xxx–xxx.`
- **Thesis:** `J. K. Author, "Title," Ph.D. dissertation, Dept., Univ., City, year.` — title in **quotes**, dissertation-type label.
- **Conference:** `J. K. Author, "Title of paper," in Abbreviated Name of Conf., …, year, pp. xxx–xxx.`
- **et al.:** list up to **six** names; **more than six → first author + "et al."** (`ieee.csl` encodes this as `et-al-min="7" et-al-use-first="1"`).

| Type | Issue | Bucket | Status |
|---|---|---|---|
| All (quoted titles) | Comma rendered *outside* the closing quote (`"Title", Journal`) though `punctuation-in-quote: true` was set | **E** | **FIXED** (engine) `9368e71d` |
| All | Entries did not end with a period | **A** | **FIXED** `fc876453` (added `entry-suffix: .`; engine suppresses for URL-terminated entries, keeps it for DOI — guide-correct) |
| All (≥7 authors) | et-al threshold was `min: 8`; guide/`ieee.csl` is 7 | **A** | **FIXED** `fc876453` |
| All (3+ names) | No serial comma before "and" (`delimiter-precedes-last: never`); `ieee.csl` defaults to **contextual** | **A** | **FIXED** (set `contextual`) |
| **Chapter** | `chapter:` only tweaked the page label, so it fell back to the journal template: no `in`, no editors/`Eds.`, wrong publisher punctuation and year/pp. order | **A** | **FIXED** — full type-variant added; now byte-identical to citeproc except the no-city `Eds.:`/`Eds.,` residual below |
| Editor label capitalisation | Guide wants `Eds.` (capitalised) but `RoleLabel` had no case option and the locale term is lowercase `eds.` | **A/E** | **FIXED** — added `text-case` to `RoleLabel` (schema + engine); IEEE chapter uses `text-case: capitalize-first` |
| **Thesis** | `thesis extends book` italicised the title and used `: University`; guide wants the title **quoted** + a thesis-type label | **A** | **FIXED** — full variant; title quoted (`emph: false`), genre + institution + year. Renders `"…," PhD thesis, Stanford University, 2019.` Guide-exact `Ph.D. dissertation` needs genre normalisation (see below) |
| Thesis genre label | Citum renders `PhD thesis` (normalised), citeproc renders raw `phd-thesis`, guide wants `Ph.D. dissertation` | **C** | OPEN — genre-string normalisation, a data concern not a style concern; Citum's output is already cleaner than citeproc's |
| **Conference** | `paper-conference:` only tweaked the page label → journal template; guide wants `… in Abbrev. Conf. Name, … pp.` | **A** | **FIXED** — full variant; now **byte-identical** to citeproc: `… "…," in *Proceedings of NIPS 2013*, 2013, pp. 3111–3119.` |
| Publisher colon with no city | When `publisher-place` is empty, chapter renders `Eds.: Publisher`; guide/citeproc render `Eds., Publisher`. Root cause is the bibliography join after an abbreviation-period (`Eds.`) emitting a space, not `, ` | **E** | OPEN — engine join-logic edge; deferred (risky to change globally) |
| Edited book (editor-as-author via `substitute`) | Renders `(eds.)` (lowercase, parenthesised); guide wants `Eds.,` | **A** | **FIXED** (csl26-h1ms) — new `short-suffix-comma` preset + `substitute.contributor-role-case`; IEEE now renders `, Eds.` |
| "et al." formatting | Citum renders `Vaswani et al. "…` (roman, no comma); citeproc/guide render `Vaswani et al., "…` (the snapshot italicises *et al.*) | **A/D** | OPEN — verify italic expectation |
| Encyclopedia entry `[1]` | Citum adds `: University of Chicago Press`; citeproc omits publisher | **A/B** | OPEN — low priority, adjudicate vs guide |
| Thesis snapshot `[11]` shows raw `phd-thesis` genre token | citeproc/data, not Citum's YAML | **C** | Note for fixture/genre follow-up |

The three OPEN structural variants (chapter/thesis/conference) are genuine
rewrites and will each be verified against the guide example **and** the oracle
snapshot before landing.

---

## APA 7th — `apa-7th.yaml`

Guide: **APA Style** (apastyle.apa.org references hub) — 7th edition.

### Metadata findings

| Field | Current | Assessment |
|---|---|---|
| `rel: documentation` | `apastyle.apa.org/style-grammar-guidelines/references` | **Current and authoritative** — the official APA references hub. OK. |
| `rel: self` / `csl-id` | `http://www.zotero.org/styles/apa` | OK (legacy `http://`, cosmetic). |
| `rel: template` | `…/apa-6th-edition` | Harmless lineage pointer; not a conformance concern. |

### Conformance — per type (Citum vs guide vs citeproc)

| Type | Issue | Bucket | Status |
|---|---|---|---|
| **Journal article** | Article title was italicised (`article-journal` variant had `title: primary emph: true`). APA 7 sets the **article title in roman**; only the journal name and volume are italic. citeproc renders it roman. | **A** | **FIXED** — `emph: false`; now matches APA 7 and the citeproc snapshot for all ~15 journal entries. (Magazine/newspaper variants were already correct.) |
| Same-surname, different-initial authors | A. Johnson and B. Johnson get spurious year-suffixes (`2020a`/`2020b`) and sort B-before-A. APA/citeproc add **no suffix**, sort by initial, and add initials in citations (`A. Johnson` / `B. Johnson`). | **A/E** | OPEN — disambiguation logic; engine concern, deferred (risky area, see [[project_disambiguation_defaults]]). |
| Legal references (cases, statutes, treaties) | `Brown v. Board of Education`, ESSA, treaties render generically; APA follows Bluebook with type-specific formats. citeproc approximates these specially too. | **B/D** | OPEN — large, specialised; out of scope for this pass. Flagged, not guessed. |
| Thesis genre token | `[PhD thesis]` (Citum) vs `[Phd-thesis]` (citeproc) vs guide `[Doctoral dissertation]` | **C** | Note — genre-string normalisation, same data concern as IEEE. Citum already cleaner than citeproc. |

The journal-title fix is YAML-only; the full engine/`bibliography` + `domain_fixtures` integration suites (84 tests) stay green — no test pinned the old italic.

---

## Chicago 18th author-date — `chicago-author-date-18th.yaml`

Guide: **Chicago Manual of Style, 18th ed.** (author-date system). The
`documentation` URL is the CMOS root (paywalled) — a fine pointer, not a
format page.

### Conformance — per type (Citum vs CMOS vs citeproc)

| Type | Issue | Bucket | Status |
|---|---|---|---|
| **Journal article** | Rendered `_Journal_. 15, (1):` — stray period after the journal name and a comma before the issue parens. CMOS/citeproc want `_Journal_ 15 (1):` (space, no comma). | **A** | **FIXED** — gave the volume/issue group `prefix: " "` (overrides the `. ` entry separator) and `delimiter: ""`. Now matches CMOS + citeproc for all journal entries. |
| **Book chapter** | Renders `"Chapter." Edited by Eds, _Book_.`; CMOS wants `"Chapter." In _Book_, edited by Eds.` (the `In _Book_` precedes `edited by`, and `In` is missing). | **A** | OPEN — structural; reorders the shared base-template `[editor, parent-monograph]` group and adds the `in` term. Deferred (touches the shared template; needs care). |
| **Magazine** | `_Wired_ 31: 42–49`; CMOS cites magazines by date (`Wired, June`), not volume/pages. | **A** | OPEN — `article-magazine` variant. |
| **Conference** | `_Proceedings of Nips 2013_: 3111–19`; citeproc `Proceedings of NIPS 2013, 3111–19`. Italic+colon vs roman+comma, and `NIPS`→`Nips` (title-case lowercasing an acronym). | **A/E** | OPEN — title-case acronym handling is an engine concern. |
| **Translator** (book) | `Translated by David Wyllie (Trans.).` — both the `Translated by` verb prefix and a redundant `(Trans.)` label. citeproc: `Translated by David Wyllie.` | **A** | OPEN — double-labelling; a default translator role-label leaks alongside the verb prefix. |
| **Patent** | `Method for Efficient Data Compression., issued July 13, 2021.` — double period, missing `Patent US …`. citeproc emits the patent number. | **A/C** | OPEN — patent variant's term/number renders empty (likely fixture field-name mismatch). |
| Same-year disambiguation order | Garcia `2019b` sorts before `2019a`; should be `a` then `b`. | **A/E** | OPEN — bibliography sort with year-suffix; engine, see [[project_disambiguation_defaults]]. |
| Legal (cases/statutes/treaties) | Generic rendering vs CMOS-Bluebook. | **B/D** | OPEN — large, specialised; same as APA. |

Metadata: documentation URL (CMOS root) OK as a pointer. Journal fix is
YAML-only; `citum-engine::document` suite (39 tests) stays green.

---

## AMA 11th — `american-medical-association.yaml`

Guide: **AMA Manual of Style, 11th ed.** citeproc confirmed to italicise
journal and book titles (Citum's italics are therefore correct — no issue).

| Type | Issue | Bucket | Status |
|---|---|---|---|
| **Journal/book DOI** | `2(2) doi:…` — only a space before `doi:`. AMA/citeproc put a period (`2(2). doi:…`). | **A** | **FIXED** — doi `prefix: ". doi:"`. |
| **Chapter/conference editor label** | `…, editors _Book_` (long form). AMA/citeproc use `…, eds. _Book_`. | **A** | **FIXED** — label `form: short`; now byte-matches citeproc. |
| Trailing period after DOI | Citum ends doi entries with `.`; citeproc has none. | **E** | OPEN — engine `entry-suffix` keeps the period for doi-ending entries (same family as IEEE url/doi handling). |
| Edited-book substitute | `Reis HT, Judd CM (eds.).`; citeproc `…, eds.`. | **A** | **FIXED** (csl26-h1ms) — AMA uses `short-comma`; renders `, eds.`. |
| Page range | `436–444` (en-dash); AMA/citeproc use a hyphen `436-444`. | **E** | OPEN — page-range formatting, engine. |

Metadata: documentation URL (`academic.oup.com/amamanualofstyle`) OK.

---

## MLA 9th — `modern-language-association.yaml`

Guide: **MLA Handbook, 9th ed.** (containers model). citeproc confirmed to
italicise container titles.

| Type | Issue | Bucket | Status |
|---|---|---|---|
| **Journal article** | `_Journal_. 4, 2019, …` — period after the container and no `vol.` label. MLA/citeproc want `_Journal_, vol. 4, …`. | **A** | **FIXED** — volume `prefix: ", vol. "` (overrides the `. ` separator and adds the label). Now matches citeproc for journal entries. |
| **Journal DOI** | `pp. 81–104. https://doi.org/…` — period before the DOI; MLA/citeproc use a comma. | **A** | **FIXED** — the `article-journal` doi modify keeps the comma (`prefix: ", https://doi.org/"`). |
| Trailing period after DOI | citeproc ends with `.`; Citum omits it (engine suppresses `entry-suffix` for url/doi-ending entries). MLA *wants* the period here. | **E** | OPEN — inverse of the IEEE case; engine `entry-suffix` policy is per-engine, not per-style. |
| Disambiguation | `2019a`/`2021a` year-suffix appears; MLA disambiguates by adding author names, not suffix letters. | **A/E** | OPEN — engine disambiguation strategy. |
| Edited book | `Reis, …, and Charles M. Judd.` — missing the `, editors` label citeproc emits. | **A** | **FIXED** (csl26-h1ms) — MLA substitute uses `long`; renders `, editors`. |
| Translator | `translated by David Wyllie.` (lowercase, period); citeproc `Translated by David Wyllie,`. | **A** | OPEN — element-initial capitalisation + delimiter. |
| Chapter / encyclopedia | `In`-ordering and missing `, vol. 5` on the container (same shape as Chicago chapter). | **A** | OPEN. |

The journal fixes required updating one golden (`document.rs` MLA plain-text
example) to the corrected `, vol. N` output; full `just pre-commit` green
(1659 tests). Metadata: documentation URL (`style.mla.org`) OK.

---

## Elsevier ×3 — `elsevier-{harvard,vancouver,with-titles}-core.yaml`

Guides: Elsevier per-journal author guides (Harvard, Vancouver-numeric, and
numeric-with-titles families). The `documentation` URLs are legacy
`http://` per-journal anchors — **flagged for browser re-verification**
(outbound HTTP is sandboxed here; could not confirm reachability).

| Style | Finding | Bucket | Status |
|---|---|---|---|
| **Harvard** | Essentially conformant — journal format (`Urban Climate 15, 1–18.`) byte-matches citeproc. Only diffs: Citum's cleaner thesis genre (`(PhD thesis)` vs `(phd-thesis)`) and a missing conference `event` note (`Presented at …`). | C/A | No fix — Citum at parity or cleaner. `event` omission noted. |
| **Vancouver** | Book chapter rendered `inEricsson KA, …` — the `in` term had no capitalisation or `: ` suffix. citeproc: `In: Ericsson KA, …`. | **A** | **FIXED** — `in` term `text-case: capitalize-first` + `suffix: ": "`; chapter now byte-matches citeproc. |
| **With-titles** | Journal arranges `Nature 521 436–444, 2015`; citeproc puts the year in parens: `Nature 521 (2015) 436–444`. Also `(eds.)` vs `(Eds.)`, and page/publisher order differs (`683–703, Publisher, 2006` vs `Publisher, 2006: pp. 683–703`). | **A** | OPEN — structural reorder (volume → `(year)` → pages); larger, deferred. |

Metadata: all three `documentation` links are legacy `http://` per-journal
anchors; re-point once browser-verified (see inventory below).

---

## Springer ×3 — `springer-{basic-author-date,basic-brackets,vancouver-brackets}-core.yaml`

Guides: Springer Basic (author-date and numeric/brackets) and Springer
Vancouver. The `documentation` URLs are **retired Springer CDN paths**
(`cda_downloaddocument`) — almost certainly dead; flagged for re-pointing
(browser-verify; sandboxed here).

| Style | Finding | Bucket | Status |
|---|---|---|---|
| **Basic author-date** | Book chapter `In Ericsson …` — `in` term missing the `:` separator. citeproc: `In: Ericsson …`. | **A** | **FIXED** — `in` term `suffix: ':'`; chapter byte-matches citeproc. |
| **Basic brackets** | (1) Same `In:` gap. (2) **Duplicate page range** `pp 683–703 683–703` — the chapter inherited the default template, which renders `pages` in both the publisher group and the volume group. | **A** | **FIXED** — `in` term `suffix: ':'`; chapter now `remove`s the volume/pages group (mirrors the author-date variant). Both byte-match citeproc. |
| **Vancouver brackets** | (1) `in. Ericsson` (lowercase, period) → needs `In:`. (2) Article/book titles sentence-cased, lowercasing the proper noun `Cambridge`→`cambridge`; citeproc preserves title case. | **A / A·D** | `In:` **FIXED** (`text-case: capitalize-first` + `suffix: ':'`, group `prefix: ' '`). Title-case OPEN — sentence-case transform should preserve proper nouns; verify against the Springer Vancouver guide. |
| Both basic (Kuhn 1962) | Extra `. University of Chicago Press` before the volume on a journal-in-series; citeproc omits the publisher there. | **A/B** | OPEN — low priority, one odd entry. |

Metadata: all three `documentation` links are retired Springer CDN paths;
re-point once browser-verified.

---

## Taylor & Francis ×3 — reviewed against the official PDFs

`taylor-and-francis-{chicago-author-date,council-of-science-editors-author-date,national-library-of-medicine}-core.yaml`.

**Guide access — now resolved.** The authoritative T&F PDFs were obtained and
read directly. Their headers correct an earlier mis-mapping (a supplied style
index was unreliable): the per-file identities are **read from the PDFs**, not
inferred —

| File | PDF header (verified) | Embedded style it backs |
|---|---|---|
| `tf_f.pdf` | Style F — Chicago Author-Date (CMOS 15th) | `…chicago-author-date` |
| `tf_c.pdf` | Style C — **CSE Name-Year** | `…council-of-science-editors-author-date` |
| `tf_nlm.pdf` | NLM Standard Reference Style v2.2 (2023) | `…national-library-of-medicine` |
| `tf_s.pdf` | Style S — **Maths** (numbered) | *(none — not CSE)* |
| `tf_n.pdf` | Style N — **British Chicago footnotes/bib** | *(none — not NLM)* |
| `tf_apa.pdf` / `tf_v.pdf` | Styles A / V — APA | *(none embedded)* |

**Metadata fixes applied:** CSE `documentation` → `tf_c.pdf` (Style C = CSE
Name-Year; supersedes the earlier wrong `tf_s.pdf`, which is the Maths guide);
NLM normalised to lowercase `tf_nlm.pdf` (the file *is* the NLM guide — the
earlier "maybe AMA" worry came from the bad index). Chicago has no `source`
block of its own (it `extends` `chicago-author-date-18th`).

**NLM — structural fixes applied** (against `tf_nlm.pdf` samples, e.g.
`Sumner P, Mollon JD. … In: Mollon JD, …, editors. Normal and defective colour
vision. New York (NY): Oxford University Press; 2003. p. 21–30.`):

| Issue | Bucket | Status |
|---|---|---|
| Missing period after author (`Kuhn TS The Structure`) — the title's `prefix: " "` overrode the `. ` separator | **A** | **FIXED** — dropped the prefix; now `Kuhn TS. The Structure …`. |
| Chapter `On edited by …` (a `prefix: 'on '` migration error + the `edited by` verb) | **A** | **FIXED** — chapter editor now `form: long`, family-first, `In: ` prefix, `, editors` label, `. ` before the book title → `In: Ericsson KA, …, editors. The Cambridge Handbook …` (matches the guide + citeproc). |
| Journal drops the year and gains a stray publisher (`2, (2): University of Chicago Press` vs `1962;2(2)`); chapter tail `2006: Publisher` vs `Publisher; 2006. p. …` | **A** | OPEN — needs the `Year;volume(number):pages` regroup and the book/chapter `Place: Publisher; Year` reorder (same shape as the AMA journal group). Larger; deferred. |

**CSE (Style C, `tf_c.pdf`) and Chicago (Style F, `tf_f.pdf`)** still carry the
same `on edited by`/`edited by` chapter shape and journal-punctuation gaps
documented previously, **plus** a guide-level finding for Style F: the guide
prescribes **sentence-case titles with no quotation marks** for article titles
(`Problems in the use of survey questions … Science 236: 957–9.`), whereas both
Citum and the citeproc CSL render Title Case *in quotes*. That is a genuine
**guide-vs-citeproc** divergence (bucket B) — adopting it is an intentional
break from the CSL reference and should be a deliberate decision, not a silent
fix. CSE/Chicago structural rewrites deferred to a focused follow-up.

---

## Metadata inventory — all embedded core styles

`documentation` URLs below were read from the YAML. Live reachability could not
be machine-verified in this environment (outbound HTTP is sandboxed); entries
are flagged by **structural** red flags (legacy `http://`, retired CDN paths)
and must be confirmed with a browser before editing. **Not yet live-verified**
means exactly that — not "dead."

| Style | csl-id | `documentation` link | Flag |
|---|---|---|---|
| IEEE | `…/ieee` | IEEE editorial-style-manual gateway | **Gateway, stale version** (see above) |
| APA 7th | `…/apa` | `apastyle.apa.org/style-grammar-guidelines/references` | Plausibly current; verify |
| Chicago author-date 18th | `…/chicago-author-date` | `chicagomanualofstyle.org/` | Root only (paywalled); fine as a pointer |
| Chicago notes 18th | `…/chicago-notes` | `chicagomanualofstyle.org/` | Root only; fine |
| AMA 11th | `…/american-medical-association` | `academic.oup.com/amamanualofstyle` | Plausibly current; verify |
| MLA 9th | `…/modern-language-association` | `style.mla.org/` | Plausibly current; verify |
| Elsevier Harvard | `…/elsevier-harvard` | `elsevier.com/journals/biological-conservation/…/guide-for-authors#68000` | **Legacy `http://`, per-journal anchor — likely moved** |
| Elsevier Vancouver | `…/elsevier-vancouver` | `elsevier.com/journals/energy/…/guide-for-authors#68000` | **Legacy `http://` — likely moved** |
| Elsevier with-titles | `…/elsevier-with-titles` | `elsevier.com/journals/journal-of-hazardous-materials/…#68001` | **Legacy `http://` — likely moved** |
| Springer Basic author-date | `…/springer-basic-author-date` | `springer.com/cda/content/document/cda_downloaddocument/…pdf` ×3 | **Retired Springer CDN path — almost certainly dead** |
| Springer Basic brackets | `…/springer-basic-brackets` | same retired CDN ×3 | **Almost certainly dead** |
| Springer Vancouver brackets | `…/springer-vancouver-brackets` | `springer.com/…/manuscript-preparation` + `dam.springernature.com/…Key+Style+Points…pdf` | Newer; verify (the `dam.springernature.com` link carries an auth query string) |
| T&F Chicago author-date | `…/taylor-and-francis-chicago-author-date` | inherits CMOS root (no own `source`; `extends` chicago-author-date-18th) | Style F guide is `tf_f.pdf`; no own `source` block to re-point |
| T&F CSE author-date | `…/taylor-and-francis-council-of-science-editors-author-date` | **`files.taylorandfrancis.com/tf_c.pdf`** (was dead `tandf.co.uk/…/tf_CSE.pdf`) | **FIXED** — Style C = CSE Name-Year |
| T&F NLM | `…/taylor-and-francis-national-library-of-medicine` | **`files.taylorandfrancis.com/tf_nlm.pdf`** (case-normalised) | **FIXED** — `tf_nlm.pdf` *is* the NLM v2.2 guide |

### Authoritative T&F guide links (verified from the official PDF headers)

`https://files.taylorandfrancis.com/<file>` — identities read from each PDF's
title block (the earlier index was wrong on Style S and the NLM file):

| File | Style / format |
|---|---|
| `tf_f.pdf` | Style F — Chicago Author-Date |
| `tf_c.pdf` | Style C — CSE Name-Year |
| `tf_nlm.pdf` | NLM Standard Reference Style v2.2 |
| `tf_s.pdf` | Style S — Maths (numbered) |
| `tf_n.pdf` | Style N — British Chicago footnotes/bibliography |
| `tf_apa.pdf` / `tf_v.pdf` | Styles A / V — APA |

(The three embedded T&F styles map to `tf_f.pdf`, `tf_c.pdf`, `tf_nlm.pdf`.)

### Cross-cutting metadata observations

- **Legacy `http://` and retired CDN paths** dominate the Elsevier and Springer
  entries. These predate Citum (inherited from the CSL styles) and should be
  re-pointed to current guide URLs once each is browser-verified.
- **Version labels** (like IEEE's "11.29.2023") embedded in `info.title` go
  stale silently. Consider whether a version belongs in the title at all.

---

## Follow-ups

The first sweep is complete; the per-style sections above record what landed and
what stays OPEN. The remaining work is tracked in three follow-up beans (detail
lives in the per-style sections, not re-duplicated in the beans):

- **`csl26-28ag`** — T&F trio structural conformance (CSE/Chicago/NLM remaining
  structure; includes the **Style-F sentence-case + unquoted-titles** decision,
  a deliberate guide-vs-citeproc divergence needing sign-off).
- **`csl26-6qv3`** — embedded style structural residuals (Chicago author-date
  chapter/magazine/conference/translator/patent, Elsevier with-titles
  year-in-parens, Chicago notes deep review).
- **`csl26-maim`** — cross-cutting render residuals (substitute `(eds.)`, the
  `entry-suffix` DOI/URL terminal-period policy, disambiguation strategy,
  proper-noun preservation under sentence-case, page-range dash).

Sweep bean (completed/archived): `csl26-53zy`.
