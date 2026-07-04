# citum-engine Crate Review

Date: 2026-07-03
Branch: `audit/citum-engine-review-2026-07`
Bean: `csl26-nj72`

## Scope and Method

Comprehensive code review of `crates/citum-engine` (~45k lines across 102
files; ~29k lines of production code excluding test modules). Review criteria:
[DESIGN_PRINCIPLES.md](../DESIGN_PRINCIPLES.md) (especially §4 "Styles Are
Declarative Contracts" and §9 "Rust Engineering Serves The Model"), the crate's
own `CLAUDE.md` contract, and general Rust API/performance hygiene.

Coverage:

- **Fully read:** `lib.rs`, `error.rs`, all of `src/api/` (document, session,
  types, style_input, refs_input, warnings, forward_compat), the processor
  spine (`processor/mod.rs`, `setup.rs`, `citation.rs`, `note_context.rs`,
  `matching.rs`, `disambiguation.rs`, `sorting.rs`, `labels.rs`,
  `rendering/mod.rs`, `rendering/grouped/core.rs`, `bibliography/mod.rs`,
  `bibliography/grouping.rs`, `document/pipeline.rs`), `ffi/mod.rs`,
  `values/mod.rs`, `render/format.rs`, `grouping/sorting.rs`.
- **Targeted scans crate-wide:** panic-path scan (`unwrap`/`expect`/`panic!`/
  `unreachable!`/`todo!`/`unimplemented!` outside `#[cfg(test)]`), `unsafe`
  scan, TODO/FIXME scan, `process::exit`/`eprintln!` scan.
- **Not read line-by-line:** `values/` submodules (date, contributor, title,
  number, locator, text_case), `render/` format backends, `document/`
  markdown/notes/djot parsers, `rendering/grouped/` template_policy and
  sentence_initial, `sort_partitioning.rs`. These were sampled and scanned
  mechanically only; a follow-up pass could cover them.

## Verification Baseline

- `cargo clippy -p citum-engine --all-targets --all-features -- -D warnings`: **clean**.
- `cargo nextest run -p citum-engine`: **846/846 passed**.
- Zero `TODO`/`FIXME`/`HACK` markers in the crate.
- No `unsafe` outside `src/ffi/` (which is `#![allow(unsafe_code)]` with
  documented `# Safety` contracts on every entry point).
- Exactly six panic-lint exceptions in production code, each carrying an
  `#[allow(..., reason = ...)]` with a defensible rationale (ICU bootstrap ×2,
  length-checked `first()` ×2, scan-loop `unreachable!` ×1, cache-hydration
  `expect` ×1 — see Finding 7 for the last one).

## Strengths

- **Lint regime is exemplary.** The workspace denies `unwrap_used`,
  `expect_used`, `panic`, `indexing_slicing`, `string_slice`, and
  `allow_attributes_without_reason`, with every noise-lint `allow` documented
  in `Cargo.toml`. The "no hidden panic paths" principle is mechanically
  enforced, not aspirational.
- **FFI is hardened.** The 2026-05-12 security audit's fixes hold: centralized
  pointer/UTF-8 parsing, thread-local last-error, null-pointer tests, no
  panics across the C ABI.
- **Module architecture matches its documentation.** `Processor` really is a
  thin facade; setup/citation/bibliography/note-context concerns are cleanly
  separated; comments consistently record *constraints* (year-suffix ordering
  vs. rendered sort order, wrap-capture semantics, borrow-release points)
  rather than narrating code.
- **Structured warnings are a real API.** `missing_ref`, `nocite_missing_ref`,
  `locale_fallback`, `unknown_reference_class/field`, `unknown_enum_variant`
  flow through one channel; forward-compat unknown-field walking
  (`api/forward_compat.rs`) is thorough and path-addressed.
- **Test posture is strong.** 846 tests, BDD naming, `rstest` parameterization,
  oracle-backed sort/i18n/multilingual suites, and boundary tests (et-al
  thresholds, label presets, ibid/locator positions).
- **Multilingual handling is principled.** Holistic name-variant selection,
  BCP 47 exact→prefix→fallback resolution, and script-aware display are data-
  driven per §3 of the design principles.

## Findings

Ordered by severity. File references are to the audit branch at the review
commit.

### High

**1. `std::process::exit(1)` inside a public library API.**
`Processor::process_document` aborts the host process on a frontmatter parse
error (`processor/document/pipeline.rs:61-64`), preceded by an `eprintln!`.
Today the only external caller is `citum-cli`, so this is latent — but the
crate contract is "one engine, multiple surfaces" and this method is the
document entry point a server/FFI/WASM surface would adopt. `exit` is worse
than a panic (no unwinding, destructors skipped, untrappable through the C
ABI), and it violates DESIGN_PRINCIPLES §9. Related: `process_document`
returns a bare `String` with no error/warning channel, and per-citation render
errors silently fall back to source text (`pipeline.rs:369,395,423`).
*Recommendation:* return `Result` (or thread the existing `Warning` channel);
move exit-on-error behavior into the CLI.

**2. Style-specific hardcoding in the engine.**
`preferred_no_date_term_form` special-cases
`csl_id == "http://www.zotero.org/styles/harvard-cite-them-right"` to select
the long "no date" term form (`processor/rendering/grouped/core.rs:1261-1274`).
This is precisely the "hidden processor magic" pattern DESIGN_PRINCIPLES §4
names as the anti-example — behavior keyed to a specific style identity
instead of declared in style data. Any style re-derived from that CSL id, or
hand-authored equivalent without the id, silently behaves differently.
*Recommendation:* promote to a typed style option (e.g.
`options.dates.no-date-term-form: long|short`), set it in the style YAML, and
delete the id match. Record the interim behavior in the divergence register if
it must ship.

### Medium

**3. Grouped-bibliography headings bypass the `OutputFormat` abstraction.**
`render_group_heading` compares `std::any::type_name::<F>()` to detect HTML
and emits Markdown `# heading` for *every other format*
(`processor/bibliography/grouping.rs:823-832`) — so LaTeX/Typst/Djot grouped
bibliographies get Markdown heading syntax. The `OutputFormat` trait already
defines `heading(level, content)` (`render/format.rs:142`). Heading policy now
lives in three places: this type-name hack, the per-format
`render_bibliography_section_heading` in `document/pipeline.rs:23-36`, and the
unused trait method. *Recommendation:* implement `heading` in each backend and
route both call sites through it.

**4. Tier-1 / session pipeline duplication.**
`DocumentSession::render_citations` (`api/session.rs:336-483`) duplicates
~130 lines of `format_document_with_style` (`api/document.rs:214-406`): locale
fallback warning, three warning scans, document-options application,
missing-ref retention, nocite registration, and the six-way
`OutputFormatKind` dispatch (which itself repeats five times across the two
files). The duplication is self-acknowledged ("mirrors Tier 1 setup"). Drift
here would produce batch-vs-interactive behavioral splits that are hard to
test for. *Recommendation:* extract a shared `prepare_processor(request) →
(Processor, Vec<Warning>, Vec<Citation>)` helper and a single format-dispatch
helper (generic fn or macro).

**5. Session mutations re-do all work.**
Every `insert/update/delete/set_nocite` re-parses the refs input from the
stored `RefsInput` (clone + `resolve_local()` — a full YAML/JSON parse),
re-runs the warning scans, rebuilds the `Processor` (including disambiguation
hint calculation over the whole bibliography), and re-renders every citation
plus the bibliography (`api/session.rs:304-483`). For the interactive use case
this API exists for, that is O(document) work per keystroke-level edit.
*Recommendation:* cache the resolved `Bibliography` (invalidate on
`put_references`), and consider caching the resolved style. Document the
current cost either way.

**6. `Processor` is an implicit state machine via `RefCell`.**
`citation_numbers`, `cited_ids`, `first_note_by_id`, `compound_groups`, and
three dynamic compound maps are interior-mutable (`processor/mod.rs:89-124`),
so `&self` render methods are order-dependent and non-idempotent: bibliography
output depends on which citations were processed first; processing the same
citation list twice can differ (dynamic groups are first-occurrence-wins).
This is by design (citeproc semantics) but the invariants live only in
comments (e.g. "must be called before `track_cited_ids_and_init_numbers`",
`processor/citation.rs:178-180`). It also makes `Processor` single-threaded.
*Recommendation:* no immediate change; longer-term, consider an explicit
per-run state object so a `Processor` can be reused/shared and the ordering
contract is typed rather than commented.

**7. Disambiguation cache keyed by reference pointer address.**
`reference_cache_key` uses `std::ptr::from_ref(reference) as usize`
(`processor/disambiguation.rs:1038-1040`) and `reference_data` `expect`s a hit
(`:1054`). This is correct today only because the cache is built and consumed
within one `calculate_hints` call over an unmoved `IndexMap`. The invariant is
non-local: any future caller that passes a cloned reference, or mutates the
bibliography between build and lookup, converts a logic change into a runtime
panic. *Recommendation:* key by reference id (falling back to map index for
id-less refs) and drop the `expect` exception.

**8. Two parallel sorting stacks; the default one is uncached.**
`Sorter` (`processor/sorting.rs`) recomputes `author_sort_key_opt` /
`title_sort_key` — including leading-article stripping and collation
normalization — for *both* operands on *every* comparison, i.e. O(n log n) key
derivations. `GroupSorter` (`grouping/sorting.rs`) already implements the
cached (Schwartzian) pattern with compiled keys. The two stacks also duplicate
`compare_optional_years` verbatim. Additionally `SortKey::CitationNumber`
compares as `Equal` — a silent no-op sort key (`processor/sorting.rs:124`).
*Recommendation:* fold `Sorter` into `GroupSorter` (map config `SortKey` to
group sort keys), and either implement or reject `citation-number` sorting
explicitly.

**9. Per-component `Config` clones in the render hot path.**
Every rendered `ProcTemplateComponent` carries `config:
Some(ctx.options.config.clone())` and a cloned `bibliography_config`
(`processor/rendering/grouped/core.rs:1092-1093, 1138-1139`), and every
`RenderOptions`/`Renderer` construction clones `BibliographyConfig`. For a
bibliography of *n* entries × *m* components that is n×m deep clones of the
full config tree per render pass. The crate has a benchmark
(`benches/rendering.rs`); this is the first place to look if it regresses.
*Recommendation:* borrow (`&'a Config`) or `Rc`/`Arc` the configs in
`ProcTemplateComponent`.

**10. Custom-groups bibliography path renders everything twice.**
`render_selected_bibliography_with_format_and_annotations` path 1 calls
`self.process_references()` — a full PlainText render of the entire
bibliography — solely to obtain entry IDs for selector matching
(`processor/bibliography/mod.rs:297`), while the sibling grouped path already
uses `sorted_id_stubs()` for exactly this (`grouping.rs:622`). Separately,
`render_document_bibliography` computes `content` and `entries` via two full
render passes (`grouping.rs:576-599`); the doc comment justifies the
consistency requirement, but one pass could produce both.
*Recommendation:* use `sorted_id_stubs()` in path 1; consider a single-pass
content+entries build later.

**11. Forward-compat template scan misses sub-spec templates.**
`unknown_enum_warnings` scans `style.templates`, `citation.template`, and
`bibliography.template` (`api/warnings.rs:145-160`) but not
`citation.integral/non_integral/subsequent/ibid` templates, `type-variants`
templates, or locale-variant templates — unknown terms/roles/date-forms there
are silently unreported, while `collect_unknown_field_paths` *does* recurse
into those specs. *Recommendation:* reuse the `walk_citation_spec` recursion
shape in the enum scanner.

**12. `RefsInput::Path` documentation promises CBOR; implementation is YAML-only.**
The variant docs say "YAML/JSON/CBOR extensions are loaded as native Citum
refs" (`api/refs_input.rs:31-32`), but `resolve_local` reads bytes through
`String::from_utf8_lossy` and parses YAML (JSON parses as a YAML subset; CBOR
cannot). Non-UTF-8 input is silently mangled by the lossy conversion instead
of erroring. *Recommendation:* correct the docs (or implement CBOR), and use
`String::from_utf8` with a proper `RefsInputParse` error.

### Low

**13. Error-type inconsistency.** `FormatDocumentError` and
`DocumentSessionError` hand-roll `Display`/`Error` impls
(`api/document.rs:109-123`, `api/session.rs:83-93`) while the crate already
depends on `thiserror` (used by `ProcessorError`). `ProcessorError` itself is
stringly — compound-set validation abuses
`ParseError("BIBLIOGRAPHY", ...)` (`processor/mod.rs:162-181`).

**14. Known substitute key routed through the `Unknown` escape hatch.**
Both `Matcher` and `Disambiguator` look up collection editors via
`ContributorRole::Unknown("collection-editor".to_string())`
(`processor/matching.rs:86`, `processor/disambiguation.rs:292-294`). If the
schema role vocabulary lacks the variant, add it; using `Unknown` as a routine
lookup allocates per call and defeats the tolerant-enum telemetry.

**15. Variable-once keys derive from `Debug` formatting.**
`get_variable_key` builds suppression keys with `{:?}` on schema enums
(`processor/rendering/mod.rs:734-756`), so renaming an enum variant silently
changes CSL variable-once dedup behavior. Prefer explicit serde names.

**16. Silent compound-set fallback.** `with_compound_sets` discards invalid
sets and builds a processor without them (`processor/setup.rs:117-122`) with
no warning emitted, while `try_with_compound_sets` errors. Per §5 "warnings
are API behavior", the lossy path should at least surface a structured
warning. Also `sort_citation_number_order` and `sort_bibliography_number_order`
are identical duplicates (`setup.rs:226-241`).

**17. Session API loose ends.** `DocumentSession::new` takes an unused
`_style_input: StyleInput` parameter (`api/session.rs:125`).
`diff_formatted_citations` reports only added/changed citations — deletions
are not represented in `affected_citations`, which adapters must infer from
the full citation list.

**18. `Renderer` exposes its internals.** `Renderer` is `pub` with all-public
fields, including the `filtered_to_original_index: RefCell<...>` scratch state
(`processor/rendering/mod.rs:29-60`). Pre-1.0 this is tolerable, but most
fields could be `pub(crate)` today.

**19. Crate documentation drift.** `crates/citum-engine/CLAUDE.md` references
`src/ffi/biblatex.rs` and `BibRefContext` (now living in `citum-refs`), sizes
`ffi/mod.rs` at ~570 lines (now 747), and the layout table names
`src/render/citations.rs` (actual file: `src/render/citation.rs`).

## Known Limitations (documented, not defects)

- Non-en-US locales cannot be resolved inside the engine; `format_document`
  emits a `locale_fallback` warning and renders en-US
  (`api/document.rs:226-243`). Adapter-side locale resolution is the declared
  follow-up.
- Gendered role labels flow through legacy `roles:` pending the MF2
  multi-selector follow-up (per crate `CLAUDE.md`).

## Recommended Follow-ups (prioritized)

1. Replace `process::exit` in `process_document` with an error return and move
   abort behavior to the CLI (Finding 1) — small change, removes the only
   process-fatal path in the crate.
2. Promote the harvard-cite-them-right no-date behavior into style data
   (Finding 2) — restores the declarative contract.
3. Route all bibliography/section headings through `OutputFormat::heading`
   (Finding 3) — fixes wrong markup for LaTeX/Typst/Djot grouped output.
4. Extract the shared Tier-1/session preparation pipeline and format dispatch
   (Finding 4).
5. Unify the two sorting stacks with cached keys (Finding 8) and cache session
   ref resolution (Finding 5).
6. Sweep the remaining Medium/Low items as individual beans; none block
   current fidelity work.

A follow-up review pass over the unread submodules (`values/date`,
`values/contributor`, render backends, document markdown/notes parsers) is
worthwhile — this review's mechanical scans found nothing alarming there, but
they contain the densest formatting logic in the crate.
