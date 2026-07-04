# citum-engine Crate Review — Part 2

Date: 2026-07-04
Branch: `audit/citum-engine-review-part2-2026-07`
Bean: `csl26-inb7`
Predecessor: [2026-07-03_CITUM_ENGINE_REVIEW.md](2026-07-03_CITUM_ENGINE_REVIEW.md)

## Scope and Method

Part 1 covered `lib.rs`, `error.rs`, `src/api/`, the processor spine,
`ffi/mod.rs`, `values/mod.rs`, `render/format.rs`, and
`grouping/sorting.rs`. This pass covers the remaining files — the densest
formatting logic in the crate — read in full, line by line:

- **values/ submodules:** `date.rs`, `contributor/{mod,names,substitute,
  labels}.rs`, `text_case.rs`, `title.rs`, `number.rs`, `locator.rs`,
  `term.rs`, `message.rs`, `range.rs`, `list.rs`, `variable.rs`.
- **render/ backends:** `html.rs`, `latex.rs`, `typst.rs`, `djot.rs`,
  `markdown.rs`, `org.rs`, `plain.rs`, `rich_text.rs`, `bibliography.rs`,
  `citation.rs`, `component.rs`.
- **processor/document/:** `markdown.rs`, `notes.rs`, `note_support.rs`,
  `djot/{mod,parsing}.rs`, `integral_names.rs`, `output.rs`, `types.rs`,
  plus the offset-consuming parts of `pipeline.rs` re-read for the
  markdown-offset finding.
- **remaining processor pieces:** `rendering/grouped/{template_policy,
  sentence_initial,component_predicates,grouping}.rs`,
  `rendering/{collapse,grouped_fallback,helpers}.rs`,
  `bibliography/compound.rs`, `sort_partitioning.rs`, `sort_support.rs`,
  `grouping/{mod,selector}.rs`, `reference.rs`.

**Sampled only:** `render/markup/*` (four small adapter files) and
`render/test_formats.rs`; test modules were skimmed for enshrined
expectations rather than audited. Finding 1 was verified by reproducing
the failure end-to-end through the CLI; other findings are from code
reading cross-checked against schema definitions, the CSL 1.0 spec, and
the divergence register (`docs/adjudication/DIVERGENCE_REGISTER.md`).

Findings already fixed on `main` since part 1 (process-exit, harvard
no-date hack, heading dispatch, sub-spec scanning, CBOR docs, id-stub
path) are not re-reported, nor are the deferred part-1 beans
(csl26-aawl, -54bk, -dog9, -wj7z, -b801, -qi7l, -wfua, -dr0r).

## Verification Baseline

- `cargo clippy -p citum-engine --all-targets --all-features -- -D
  warnings`: **clean**.
- `cargo nextest run -p citum-engine`: **852/852 passed**.
- Panic-lint `#[allow]` sweep over the reviewed files: all string-slice
  and indexing exceptions carry accurate reasons and are byte-boundary
  safe (verified for the hand-rolled markdown/djot scanners, the NUL
  token remapper, and the note punctuation mover) — with one systemic
  exception: the "parser-guaranteed boundaries and indices" reason on
  the document splicing paths is *not* guaranteed for the Markdown
  parser (Finding 1).

## Strengths

- **The hand-rolled scanners are byte-safe.** Every `string_slice`
  allow in `document/markdown.rs`, `djot/parsing.rs`, and
  `note_support.rs` slices at offsets derived from `find()` on 1-byte
  ASCII delimiters or `char_indices`; multibyte prose cannot split a
  boundary. The NUL-token → HTML-comment remapping around
  pulldown-cmark is careful and well-documented.
- **Punctuation-in-quote is systematically tested.** The "full monty"
  matrix in `render/citation.rs` pins the entire terminal-punctuation
  collision table for plain and Typst outputs, and
  `render/bibliography.rs` mirrors the same rules on the entry path.
- **`.nocase` protection is structural.** Case transforms run as Djot
  text-leaf transforms (`rich_text.rs`), so protected spans and markup
  survive by construction rather than by regex.
- **Note-placement logic is principled.** `note_support.rs` models
  punctuation/number/order rules as data (`NoteRule`), handles
  quote-adjacent punctuation migration in both directions, and the
  French/English defaults are overridable via `options.notes`.
- **Sort support is honest about ICU.** `sort_support.rs` documents
  exactly what the collator does (secondary strength, shifted
  punctuation) and tests NFC/NFD, case, script, and punctuation
  ignorability explicitly.
- **Integral name-memory is cleanly factored.** First/Subsequent state,
  body-vs-note contexts, and chapter/section scoping in
  `integral_names.rs` are small, pure, well-named functions.

## Findings

Ordered by severity. Line numbers are as of the review commit.

### High

**1. Markdown documents with frontmatter panic the engine.**
`MarkdownParser::parse_document` stores *absolute* citation offsets
(`start: body_start + start`,
`processor/document/markdown.rs:103-139`), while `DjotParser` stores
*body-relative* offsets (`document/djot/mod.rs:44-52`). The pipeline
assumes the djot convention: it slices the body
(`pipeline.rs: let body = &content[parsed.body_start..]`) and then
indexes that body with the parsed offsets
(`pipeline.rs:401` and the other splicing loops in `pipeline.rs` and
`notes.rs`). Reproduced:

```
$ citum render doc doc.md --input-format markdown -b refs.yaml -s apa-7th
thread 'main' panicked at crates/citum-engine/src/processor/document/pipeline.rs:401:37:
end byte index 152 is out of bounds for string of length 36
```

where `doc.md` is any Markdown document with a YAML `---` frontmatter
block. When the frontmatter is short enough that the shifted offsets
stay in range, the output is silently corrupted (prose duplicated
around the splice point) instead of panicking. This violates the
crate's no-panic contract and invalidates the "parser-guaranteed
boundaries" allow-reasons on every splicing loop. It is currently
masked because the CLI defaults to `--input-format djot` and no test
combines Markdown input with frontmatter.
*Recommendation:* make `MarkdownParser` emit body-relative offsets
(delete the `body_start +` adjustments; `footnote_placement` ranges
included), add a frontmatter+markdown round-trip test, and consider a
debug assertion that `citation.end <= body.len()` at the splice sites.

**2. Ungated template rewriting and entry suppression for anonymous
entries.** `apply_anonymous_entry_bibliography_policy`
(`processor/rendering/grouped/template_policy.rs:55-137`) runs on
*every* bibliography entry (`grouped/core.rs:838`) with no style
option gating it. For `entry-dictionary`/`entry-encyclopedia`/
`chapter` references with no visible author it (a) rewrites the
style's template into a hardcoded container-led component order, or
(b) — for entries without a DOI/URL — returns `None`, which **drops
the entry from the bibliography entirely** (`SuppressPrintLike`).
The `chapter` gate is a shape heuristic: "template contains a
`version` variable" (`template_has_dictionary_entry_shape`). This is
the DESIGN_PRINCIPLES §4 anti-pattern: Chicago's "well-known reference
works are cited in notes only" convention baked into the engine and
imposed on every style. Failure scenario: a hand-authored APA-family
style with an anonymous print encyclopedia entry silently loses the
entry from the reference list; adding an unrelated `version` variable
to a chapter template changes which chapters render.
*Recommendation:* promote to a declared bibliography option (like the
adjacent `article_journal.no_page_fallback`, which is correctly
config-gated), default it off, and record the behavior in the
divergence register.

**3. HTML backend emits reference data unescaped.** `Html::text`
returns the string verbatim — "we avoid escaping and use raw Unicode"
(`render/html.rs:48-51`) — so `<`, `>`, `&` in titles, names, or
locators pass straight into HTML output: a reference titled
`Design & use of <T> arenas` produces broken markup, and a hostile
reference (`<script>…`) is an XSS vector for any server/FFI consumer
that trusts engine output. The backend is inconsistent with itself
(attribute values *are* escaped via `escape_attribute_value`; hrefs
are sanitized) and with the Markdown backend, which escapes all active
characters. `Html::citation` also interpolates raw user-supplied ids
into `data-ref="…"` without attribute escaping
(`render/html.rs:154-160`). The Djot and Org backends have the same
no-escape policy for their own metacharacters (`_`, `*`, `~`), which
at minimum mangles formatting when data contains them.
*Recommendation:* escape `&`, `<`, `>` in `Html::text` and the
`data-ref` ids; decide and document an explicit policy for Djot/Org
metacharacters.

### Medium

**4. Sentence- and title-case transforms destroy mixed-case words.**
`to_sentence_case` lowercases the whole string before capitalizing
(`values/text_case.rs:114-120`), and `to_title_case` lowercases every
word before re-capitalizing (`text_case.rs:311-373`). CSL 1.0 and
citeproc-js preserve mixed-case words in both transforms ("An
Introduction to DNA" → sentence "An introduction to DNA"; title case
leaves "DNA", "McDonald", "iPhone" intact). Citum renders "An
introduction to dna" and "The Dna of Empire" — the latter literally
asserted in `test_title_case_structured` (`text_case.rs:629-634`).
`.nocase` spans protect marked data, but CSL-derived corpora are not
nocase-annotated. This divergence is not in the divergence register.
*Recommendation:* only lowercase words that are not already
mixed-case/uppercase (per CSL), or register the "nocase-or-flatten"
model as an intentional divergence and document it for migrators.

**5. Date ranges bypass locale date patterns and month forms.**
Single dates resolve `pattern.date-*` locale patterns
(`values/date.rs:605-756`), but ranges use `format_range_start`
(`date.rs:325-430`) — a ~100-line near-duplicate of
`format_single_date` *without* pattern resolution — and
`extract_range_end` (`date.rs:207-262`), which hardcodes English
`"{month} {d}, {year}"` assembly and always uses long month names even
for the abbreviated forms. Failure scenario: es-ES full-form single
date renders "12 de enero de 2023", but the same style rendering the
range `2023-01-12/2024-01-15` gets "enero 12, 2023–enero 15, 2024";
a `day-month-abbr-year` range renders a short-month start and a
long-month end. *Recommendation:* route both range endpoints through
`format_single_date` (year-suppressed variant for same-year ends) and
delete the duplicate.

**6. EDTF seasons never render; locale season terms are dead.**
`citum_edtf::Edtf::month()` returns `None` for
`MonthOrSeason::Season`, `extract_month` renders it as empty, and
`extract_range_end` matches only `Month` (`values/date.rs:33-43,
226-229`). No engine code reads `locale.dates.seasons`, although the
schema defines it and en-US ships four season names
(`citum-schema-style/src/locale/types.rs:369`). Failure scenario:
`issued: 2023-21` (Spring 2023) silently renders as "2023" in every
form, where citeproc-js renders "Spring 2023". *Recommendation:*
map seasons 21-24 through `locale.dates.seasons` wherever months are
resolved, or emit a structured warning while unsupported.

**7. Undated works never receive year-suffix disambiguation.**
The no-date path in `TemplateDate::values` returns the "n.d." term
before the disambiguation suffix is computed
(`values/date.rs:774-800`), and `compute_disamb_suffix` requires a
non-empty year (`date.rs:494-520`). Disambiguation grouping *does*
group undated same-author works (the group key is `author:` with no
year, `processor/disambiguation.rs:927-944`), so the hints exist but
are unusable. Failure scenario: two undated works by the same author
in APA both render "(Smith, n.d.)" — indistinguishable — where
citeproc-js renders "n.d.-a"/"n.d.-b". *Recommendation:* apply the
suffix to the no-date term via the locale's year-suffix pattern
(APA uses "n.d.-a"; make the joiner a locale/style-controllable
affix).

**8. Et-al joins ignore the configured name delimiter.**
`apply_et_al` hardcodes `", "` before "et al." when
`delimiter_precedes_et_al` applies and hardcodes `" … "` for
et-al-use-last (`values/contributor/names.rs:225,250-254`), while the
configured `delimiter` is used everywhere else. Failure scenarios: a
style with delimiter `"; "` renders "Smith; Jones, et al."; APA-7
21-author entries render "…, Barnacle, B. … Zebra, Z." — citeproc-js
places the name delimiter before the ellipsis ("…, Barnacle, B., …
Zebra, Z."). Neither shape is covered by a test.
*Recommendation:* use `et_al.delimiter` for both joins; add oracle
coverage for et-al-use-last.

**9. `delimiter-precedes-last` is partially ignored.**
In citation context the two-name branch hardcodes "never"
(`values/contributor/names.rs:154-157`), overriding an explicit
`delimiter-precedes-last: always`; in bibliography context a
`GivenFirst` name order forces "never" (`names.rs:139-141`); and
`Contextual` defaults to *true* for two names in bibliographies
(`names.rs:146`) where the CSL definition of contextual is "three or
more names". These are plausible house-style heuristics but they are
spec divergences that silently override declared style options, and
none is in the divergence register. Failure scenario: a style
declaring `delimiter-precedes-last: always` renders "(Smith and
Jones, 2020)" instead of "(Smith, and Jones, 2020)".
*Recommendation:* honor the declared option in all branches and move
the context defaults into schema-level defaults.

**10. `TemplateNumber.form` (ordinal/roman) is silently ignored.**
The schema defines `form: numeric|ordinal|roman` on number components
and documents ordinals as the point of `number:`
(`citum-schema-style/src/template.rs:917-947,1080-1090`), but
`TemplateNumber::values` never reads `self.form`
(`values/number.rs:154-213`) and no engine code references
`NumberForm`. A style declaring `number: edition, form: ordinal`
renders "2" where "2nd" is expected — with no warning. The `gender`
field is similarly advertised "for number/ordinal agreement" but can
only affect label terms. *Recommendation:* implement ordinal/roman
rendering (locale ordinal suffixes exist in CSL locales) or reject the
option loudly at style load.

**11. Page/locator range formats disagree with each other and with
CMOS.** Three separate defects:
(a) `format_chicago` renders 101-108 as "101–08"
(`values/number.rs:336-363`, asserted in the test at `number.rs:387`)
where CMOS 17 and citeproc-js render "101–8" (single changed digit for
start values 101-109); `Chicago16` is aliased to the same behavior.
(b) The *locator* path's `apply_range_format`
(`values/locator.rs:218-256`) implements `Minimal`/`MinimalTwo` as
"strip shared prefix only when the input end is already abbreviated" —
a full range "321-328" with `range-format: minimal` renders unchanged
— and `Chicago*` falls through to expanded ("simplified"). The same
option name thus formats differently for `pages` fields vs locators.
(c) Labeled locator segments default to `PageRangeFormat::Expanded`,
ignoring the config's `range_format` (`locator.rs:170-172`), while
unlabeled segments in the same pattern honor it (`locator.rs:86-88`).
*Recommendation:* share one range-format implementation between
number.rs and locator.rs, fix the CMOS 101-109 rule, and register any
retained simplification.

**12. Locale quote terms are never consulted.** The locale schema
carries `open_quote`/`close_quote`/`open_inner_quote`/
`close_inner_quote` (`citum-schema-style/src/locale/types.rs:454-463`),
but rendering uses hardcoded English marks everywhere:
`unicode_quote_marks` (`render/format.rs:14`), each backend's
`quote()`/`wrap_punctuation()` (e.g. `render/html.rs:94`,
`render/plain.rs`), and LaTeX's `` ``…'' `` pair
(`render/latex.rs:75-89`). Failure scenario: a fr-FR style whose
locale declares guillemets renders “…” in citations, bibliography, and
smart-quoted titles alike. *Recommendation:* thread the active
locale's quote terms through `quote_marks(depth)`; keep backend
hardcoding only as fallback.

**13. Punctuation-boundary logic understands HTML only.**
`visible_text` strips `<…>` tags (`render/bibliography.rs:26-40`) and
underpins all separator/dedup decisions
(`component_starts_new_sentence`, `append_rendered_component`,
`ends_with_sentence_ending_visible_punctuation`), and
`push_delimiter`'s collision table inspects the raw last char
(`render/citation.rs:72-98`). For LaTeX/Typst/Markdown, markup
wrappers hide terminal punctuation: a component rendering
`\emph{Title.}` ends in `}`, so the period-separator is appended and
LaTeX output shows "Title.. Next" where HTML output correctly shows
one period. This breaks DESIGN_PRINCIPLES §7 (backends may differ only
in markup, not in citation logic). Relatedly,
`cleanup_dangling_punctuation` (`bibliography.rs:386-416`) runs a
global find/replace loop (`".."`→`"."`, `"  "`→`" "` …) over the
*entire marked-up entry*, including attribute values and data content
— the comment history shows one pattern already had to be removed for
eating author initials. *Recommendation:* give `OutputFormat` a
"visible text" / "logical last char" hook (or track logical boundaries
in `ProcTemplateComponent`), and constrain the cleanup pass to
text outside markup.

**14. Engine-hardcoded per-type/per-value presentation rules.**
A cluster of §4 violations, each small but compounding:
- `get_effective_rendering` rewrites a suffix when
  `ref_type == "dataset"`, the value starts with `[`, *and* the suffix
  is the literal English string `" [Dataset]."`
  (`render/component.rs:270-283`) — style- and language-specific
  behavior keyed on a string literal in engine code.
- `parent_short_title` gates `ParentSerial` on
  `ref_type().contains("article")` (`values/title.rs:126`) — matching
  any future type containing "article" — while sibling functions use
  proper `ClassExtension` matching.
- `type_class_matches` hardcodes legal/classical type lists including
  `ref_type.contains("ancient")` (`values/locator.rs:285-306`).
- `get_title_category_rendering` embeds "Legacy hardcoded logic" type
  tables (`render/component.rs:315-330,369-393`) that differ from the
  other lists.
- `SimpleVariable::Url` synthesizes a DOI URL only for
  `ref_type == "dataset"` (`values/variable.rs:222-226`).
- `aliased_type_selector_candidates` makes `chapter` silently match
  `entry-dictionary` type-variants
  (`rendering/grouped/component_predicates.rs:33-38`).
Failure scenario: renaming/adding a reference type changes rendering
in six inconsistent places; a style's `type-variants` fire for types
the author never listed. *Recommendation:* centralize type
classification (one mapping table in schema or a single engine
module), and replace the `[Dataset]` literal hack with a schema
option.

**15. The two document parsers disagree on citation syntax, and both
fail silently.** Djot keys stop at `.` and `:`
(`document/djot/parsing.rs:287`: alphanumeric/`_`/`-` only) while
Markdown accepts `:` and `.` (`document/markdown.rs:410-421`); djot
`[@smith:2020]` silently cites "smith" and discards ":2020" (the
unconsumed remainder of the bracket is dropped by `repeat`). Djot has
no prefix/suffix support (`[see @kuhn]` is silently not a citation);
Markdown supports prefixes but rejects a whole bracket cluster when
items mix suppress-author states, silently leaving `[@a; -@b]` as
literal text (asserted in
`test_unsupported_bracket_cluster_does_not_fall_back`,
`markdown.rs:574-580`), where Pandoc treats suppression per-item.
`find_citations` in both parsers scans raw text, so `[@key]` inside
fenced code blocks or inline code spans is parsed and replaced.
*Recommendation:* unify the key charset (superset), support per-item
suppression, surface a document warning for dropped/malformed
citation candidates, and skip code spans (both parsers already run
pulldown-cmark/jotdown for footnotes — reuse those events to mask
code ranges).

**16. Author-substitute titles are quoted by fiat; long-form roles get
auto-labels.** Two engine-level presentation defaults that styles
cannot see or disable declaratively:
(a) `resolve_title_substitute` wraps the substituted title in quotes
in citation context unconditionally
(`values/contributor/substitute.rs:489-492`) — an authorless *book*
cited by title in an author-date style renders “Title” where APA and
citeproc-js italicize it (per-type title formatting is bypassed).
(b) With no label config at all, `resolve_role_labels` appends
" (ed.)"-style suffixes for seven hardcoded roles in `Long` form
(`values/contributor/labels.rs:320-345`), so a bare
`contributor: editor` component cannot render without a label except
via `role.omit`. Also `resolve_explicit_label` recognizes only the
term keys "chair"/"editor"/"translator" and silently substitutes the
component's own role term for any other string
(`labels.rs:210-215`). *Recommendation:* drive substitute-title
formatting through the normal title-category rendering; move the
form-based label defaults into schema defaults; warn on unknown label
term keys.

### Low

**17. LaTeX URL and symbol escaping gaps.** `Latex::link` interpolates
the URL into `\href{…}` unescaped (`render/latex.rs:119-121`): a DOI
URL containing `%` comments out the rest of the line; `#` breaks
grouping. `Latex::finish` rescues only bare `&` from non-`text()`
content paths (`latex.rs:42-57`); locale terms or delimiters
containing `_`, `%`, `~` would break identically. Wrap `\href` targets
with percent/hash escaping (or `\url`-style verbatim), and consider
routing all locale-term content through `text()`.

**18. Plural-label heuristic over-fires on identifier variables.**
`check_plural` treats any `-`, `–`, `,`, `&` as plural
(`values/number.rs:251-255`) and `number_var_to_locator_type` maps
docket/patent/standard/report numbers to `LocatorType::Number`
(`number.rs:231-236`), so `docket-number: "19-1392"` with a label
renders "nos. 19-1392". Restrict range detection to numeric-ish
values or exempt identifier-like variables.

**19. Inverted-name suffixes join with a space.**
`assemble_inverted_long_name` appends the name suffix with `' '`
(`values/contributor/names.rs:606-635`), rendering "Smith, J. Jr."
where citeproc-js renders "Smith, J., Jr." (sort-separator before the
suffix). No test covers suffixed names; add oracle coverage before
changing.

**20. Frontmatter delimiter scan is not line-anchored.**
`parse_frontmatter` opens on `starts_with("---")` and closes on the
*first* `---` substring (`document/djot/parsing.rs:376-397`), so a
document starting with a `----` thematic break is treated as
frontmatter, and a YAML value containing `---` truncates the block
(now a loud error, but a confusing one). Anchor both delimiters to
line starts.

**21. Note-style rules and duplication in notes.rs.**
`locale_note_rule` hardcodes en-US/fr punctuation-placement rules in
engine code rather than locale data
(`processor/document/notes.rs:542-567`) — overridable via
`options.notes`, but the defaults belong in locale files.
`process_note_document` / `process_note_document_html` and
`prepare_note_citations` / `_html` are ~70-line near-verbatim
duplicates (`notes.rs:28-97,104-173,212-309`); the per-citation
render-error fallbacks to raw source text (part-1 finding 1's
"related" item) still exist on these paths.

**22. Assorted silent fallbacks that merit warnings.**
- A missing locale term renders the component as nothing
  (`values/term.rs:26-30` `unwrap_or_default`), with no
  `unknown_term`-style warning at render time.
- `SelectorEvaluator::matches_field` returns `false` for any field
  name other than `language`/`note` (`grouping/selector.rs:117-135`)
  — a selector on `keywords` silently never matches.
- Numeric-collapse output hardcodes `–`, `-`, and `,`
  (`processor/rendering/collapse.rs:81,175-185`) with no
  locale/style hook.
- `contributor_role_to_reference_role` routes nine known template
  roles through `DataRole::Unknown(String)` allocations
  (`values/contributor/mod.rs:50-83`) — the same pattern csl26-dr0r
  tracks for collection-editor; that bean's scope should widen to the
  full list (chair, inventor, counsel, container-author,
  editorial-director, textual-editor, original-author,
  reviewed-author).
- `format_role_term` honors only `text-case: capitalize-first` on
  role terms; other declared transforms are silently dropped
  (`values/contributor/mod.rs:137-142`).

## Known Limitations (documented, not defects)

- `NegativeUnspecifiedYears::Fuzzy` is a documented no-op ("falls back
  to range if selected") — the unused parameter in
  `format_display_year` is intentional.
- The `Genre` variable suppresses values that merely restate the
  reference type; the rationale is documented inline
  (`values/variable.rs:237-245`).
- Plain-text emph/strong render as pseudo-markup (`_…_`, `**…**`) by
  design.

## Triage (2026-07-04)

Every finding has a disposition: fixed on this branch (PR #1002) or
deferred to a bean. Nothing is untracked.

| Finding | Severity | Disposition |
|---|---|---|
| 1 — Markdown frontmatter offset panic | High | fixed: `a32d3a5a` markdown offsets are body-relative |
| 2 — Ungated anonymous-entry rewrite/suppression | High | fixed: `57143fba` gate anonymous-entry bib policy (option `bibliography.options.anonymous-entries`; divergence `div-010`) |
| 3 — HTML output unescaped | High | fixed: `f28f931c` escape html text output and data-ref; Djot/Org metachar policy deferred: [[csl26-ejaf]] |
| 4 — Case transforms flatten mixed-case words | Medium | fixed: `88dbbc0f` preserve mixed-case words in casing (CSL semantics) |
| 5 — Date ranges bypass locale patterns | Medium | deferred: [[csl26-k6ty]] |
| 6 — EDTF seasons never render | Medium | deferred: [[csl26-3m45]] |
| 7 — No year suffixes for undated works | Medium | deferred: [[csl26-ebs3]] |
| 8 — Et-al joins ignore configured delimiter | Medium | fixed: `19f8abf0` honor delimiter in et-al joins (use-last shape verified against citeproc-js) |
| 9 — delimiter-precedes-last partially ignored | Medium | deferred: [[csl26-mc0c]] (oracle-first batch) |
| 10 — NumberForm ordinal/roman ignored | Medium | deferred: [[csl26-c361]] |
| 11 — Range formats disagree; CMOS rule | Medium | deferred: [[csl26-2ubj]] |
| 12 — Locale quote terms dead | Medium | deferred: [[csl26-o33x]] |
| 13 — Punctuation boundaries HTML-only | Medium | deferred: [[csl26-ztxq]] |
| 14 — Hardcoded per-type presentation rules | Medium | deferred: [[csl26-92mg]] |
| 15 — Parser syntax parity + silent drops | Medium | deferred: [[csl26-esq8]] (also covers the `@key.` trailing-punctuation greediness found while fixing 1) |
| 16 — Substitute-title quoting; auto role labels | Medium | deferred: [[csl26-mc0c]] (oracle-first batch) |
| 17 — LaTeX href escaping gaps | Low | fixed: `080648f8` escape latex href targets; broader `finish()` policy folded into [[csl26-ejaf]] scope decision |
| 18 — Plural-label heuristic over-fires | Low | deferred: [[csl26-4l5t]] |
| 19 — Inverted-name suffix join | Low | deferred: [[csl26-mc0c]] (oracle-first batch) |
| 20 — Frontmatter delimiter scan | Low | fixed: `4029bb1a` anchor frontmatter to line starts |
| 21 — Note rules hardcoded; notes.rs duplication | Low | deferred: [[csl26-boql]] |
| 22 — Assorted silent fallbacks | Low | deferred: [[csl26-ol1j]]; Unknown-role list widened in [[csl26-dr0r]] |

## Recommended Follow-ups (prioritized)

1. Fix the Markdown offset base and add a frontmatter+markdown test
   (Finding 1) — a reproducible panic in a supported input path.
2. Gate or remove the anonymous-entry template rewrite/suppression
   (Finding 2) — silent bibliography data loss.
3. Escape HTML text output and `data-ref` attributes (Finding 3).
4. Decide the case-transform contract (Finding 4) and either fix to
   CSL semantics or register the divergence — this silently affects
   every acronym in every sentence/title-cased style.
5. Unify range formatting (Findings 5, 11) and implement or reject
   `NumberForm` (Finding 10) — declared options that currently lie.
6. Sweep the delimiter/et-al divergences (Findings 8, 9) with oracle
   fixtures before changing behavior.
7. File the remaining Medium/Low items as beans; none block current
   fidelity work except where a style wave touches the affected
   feature (seasons, locale quotes, note styles in Markdown).
