# Group Select Specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-09-08
**Supersedes:** None
**Related:** `docs/specs/RENDER_WHEN_CONTRACT.md`,
`docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md`,
`csl26-x79y`, `csl26-zs9y`, `csl26-2hr4`, `csl26-zmxt`

## Purpose

`TemplateGroup` gains a `select` field: `all` (the default, today's
unchanged behavior — render every non-suppressed child and join them) or
`first` — render the first child that produces non-empty output, and
discard the rest. No condition, no field-presence test: just "try this,
and if it's empty, try the next thing."

T&F-CSE's publisher place is a good example of what `select: first`
covers: *print the publisher's place; if the reference doesn't have one,
print "[place unknown]" instead.* Today that needs two `render-when`
groups keyed on the same field in opposite directions. With `select:
first` it's one group:

```yaml
- group:
  - variable: publisher-place
  - message: term.place-unknown
  select: first
```

The companion decision record
(`docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md`) found 49
uses of `render-when` across the style corpus that are this same shape —
"try one rendering, and failing that, another" — and none of them actually
test anything a field-presence probe is suited for. `select: first` is the
declarative primitive for that shape. The schema already has three
narrower versions of it — `Substitute.candidates` (contributor fallback),
`DateFallbackCandidate` (issued-date fallback), `ArticleJournalNoPageFallback`
(one hardcoded DOI fallback) — see "Relationship to existing mechanisms"
below for why this doesn't replace them.

**Why a mode on `group:` and not a new template primitive:** an earlier
draft proposed a separate `alternatives:` component. It required its own
placement restriction (several parts of the engine that locate titles,
contributors, dates, etc. by walking the template structurally would not
have seen inside it) and its own "did this render" rule, which then
disagreed with `group:`'s existing term-only-content suppression in a
way that forced an awkward authoring rule (a term-only fallback candidate
had to be a bare leaf, not wrapped in `group:`). `select: first` is still
a plain `TemplateComponent::Group`, so every one of those consumers
already sees inside it with no changes. The two competing suppression
rules mostly don't arise, because `select: first` itself doesn't check
"is this whole thing backed by real data" — a term-only candidate works
with no special case (see Evaluation). They do arise at exactly one
boundary, though: a `select: first` group nested *inside* a `select:
all` parent's term-only-content gate (`is_term_only_component`, which
judges a nested group's term-only-ness structurally — "are all its
children term-only" — with no notion of which child would actually win).
See the Implementation Notes for the fix
(`Renderer::effective_term_only_component`).

## Scope

In scope: the `select` field, evaluation order for `select: first`, and
what counts as "rendered non-empty" in that mode.

Out of scope:

- any condition on source data — that stays `render-when`'s job, frozen at
  its current vocabulary (`RENDER_WHEN_CONTRACT.md`);
- style rules that pick a branch based on a property that never appears in
  the output (e.g. Chicago's editor form depends on whether a volume is
  numbered, not on rendering the number) — a genuinely different problem,
  tracked as "work-form routing" under `csl26-zmxt`;
- migrating `Substitute` or `DateFallbackCandidate` onto `select: first` —
  both carry domain-specific behavior a bare candidate list doesn't model.

## Design

### Evaluation

1. Under `select: first`, children are tried in order.
2. A child "renders" under the exact rule that already governs its own
   kind today — nothing new. Unlike `select: all`, there is no group-level
   "is this backed by real data" gate: each child is judged only on
   whether *it* produces non-empty text. A term-only message as a direct
   child works with no special authoring rule, because "does this specific
   candidate produce output" is exactly what a locale message answers by
   rendering its own text.

   **Direct consequence:** if every candidate in a `select: first` group
   is a term/message with no data dependency, the *first* one always wins
   and everything after it is unreachable — the group degenerates to
   "just write that first term directly." This isn't a new failure mode;
   it's the same "an unconditionally-rendering candidate placed first
   starves the rest" risk any ordered fallback has (see the rejected
   NLM-DOI example below, which hits the identical issue with `date:
   issued` as a first candidate). Not caught by validation — an authoring
   concern to flag in review, not a schema rule.
3. The first child that renders wins. Its output, prefix, and suffix are
   used as-is; later children are never evaluated. Trying (and
   discarding) a candidate must not leave any trace — it must not mark a
   variable as used, consume a role, or otherwise affect the winner or
   anything rendered afterward. Each candidate is evaluated against a
   cloned copy of that state; only the winner's mutations are kept.
4. If no child renders, the group renders nothing — same as `select: all`
   with no meaningful content.

`term.place-unknown` above is illustrative — that locale term doesn't
exist yet and would need adding via the normal locale-authoring path
before a style could actually use this example.

### Validation

`select: first` is rejected when the group has fewer than two children (no
fallback behavior to express — write the single component directly) or
when combined with `delimiter` (meaningless when only one child ever
contributes output).

### Relationship to existing mechanisms

`Substitute.candidates`, `DateFallbackCandidate`, and
`ArticleJournalNoPageFallback` each work as a named option because they
anchor to exactly one semantic slot every reference has, regardless of
style — *the* contributor position, *the* issued date. `select: first`'s
49 target uses have no such shared slot: `volume-title` only matters
inside one Chicago shape, `publisher`, `recipient`, and `archive-location`
each guard a different, unrelated position in a different template. An
options table keyed by field name would have to smuggle the whole template
position (type-variant, delimiter, emphasis) back in through the option,
at which point it's a template fragment pretending to be an option. Living
on `group:` directly lets a style author add a new fallback with no schema
change at all — see the decision record for the full comparison.

**Rejected: Chicago's `volume-title` position as a worked example.** It
looks like a two-way swap but isn't: the literal YAML
(`chicago-author-date-18th.yaml:416-430`) has a third, interacting group —
`field-present: part-number-non-numeric` also renders `title: primary`,
overriding `volume-title` even when it's present. `select: first` has no
way to make a candidate conditionally inapplicable other than by
rendering it and checking for output, so a present `volume-title` always
wins, contradicting the real rule. Chicago's volume-title position stays
`render-when` and is a `csl26-zmxt` (work-form routing) candidate.

NLM's DOI-when-no-page-or-volume rule looks similar but isn't `select:
first`-shaped either: it's a real, declared condition ("if this type and
these fields are absent") evaluated before anything renders, not an
output-based fallback — and its normal detail block includes `date:
issued`, present on nearly every reference, so an output-based fallback
using it as the first candidate would never fall through to DOI. Extending
`ArticleJournalNoPageFallback` is the right fix, tracked separately in
`docs/specs/MEDIUM_DESIGNATOR.md`.

## Implementation Notes

*(for engineers implementing this spec — not required reading to
understand the design)*

- Schema: `TemplateGroup.select: TemplateGroupSelect`
  (`crates/citum-schema-style/src/template.rs`, alongside `render_when`/
  `delimiter`), `#[serde(default)]`, `TemplateGroupSelect { #[default]
  All, First }` (kebab-case wire form). No new `TemplateComponent`
  variant — `dispatch_component!` (`macros.rs`) needs no change.
- Evaluation: a mode branch in `render_group_component_with_format`/
  `render_group_child_values`
  (`crates/citum-engine/src/processor/rendering/grouped/core.rs`) — when
  `select == TemplateGroupSelect::First`, use a new sibling loop (clone
  the tracker per child, early-exit on the first non-empty result, merge
  only the winner's delta back); `All` keeps today's loop, which shares
  one tracker across all children and renders every one.
- **Blocking prerequisite (`csl26-2hr4`):** `render_group_component_with_format`
  merges a group's tracker mutations into its parent *before* checking
  whether the group actually rendered anything, at every nesting depth.
  A losing `select: first` candidate that is itself a nested group
  containing a suppressed sub-group can leak that sub-group's tracker
  state before the outer `select: first` gets a chance to discard it.
  Fix the ordering bug in `render_group_component_with_format` itself
  before implementing `select: first`. **Update (2026-09-08, csl26-2hr4
  landed):** the tracker actually holds two different kinds of state.
  `rendered_vars`/`substituted_bases` (variable-once marks, substitution
  bookkeeping) can only mutate on a component's success path, so they
  cannot leak from a suppressed/empty group under current code — no
  constructible regression exists for `select: all` today, confirmed by a
  zero-diff `report-core.js` run across the embedded corpus.
  `issued_occurrences` (the date-fallback occurrence counter) mutates
  unconditionally and turns out to be *intentionally* unconditional — an
  existing test (`blank_nested_first_issued_still_advances_the_later_lane`)
  proves a blank/suppressed `date: issued` must still occupy its
  structural position in fallback-lane ordering. The landed fix splits the
  merge accordingly (`issued_occurrences` still advances unconditionally;
  the rest merges only when the group rendered) — a defensive ordering fix
  that `select: first`'s isolation depends on, not a correction to
  observable `select: all` behavior.
- No arm is needed in `TemplateResourceBudget::check_component`
  (`style/validation.rs:490`), `template_policy.rs`, or the leading-affix
  helpers (`rendering/helpers.rs`) — all of them already recurse into any
  `TemplateComponent::Group` regardless of `select`, since it's the same
  variant either way. This is most of the payoff of using a mode on
  `Group` instead of a new component type: almost no consumer audit, no
  content restriction.
  **Update (2026-09-08):** `is_term_only_component`
  (`component_predicates.rs:61`) is the one exception, found via a
  `report-core.js` fidelity drop while migrating the T&F-CSE worked
  example below. It judges a `Group` structurally ("are all children
  term-only"), which is right for `select: all` but wrong for `select:
  first`: a `select: first` group's *winner* — not its full child
  list — decides whether the group counts as term-only to an enclosing
  `select: all` parent's "backed by real data" gate. Concretely: `{select:
  all, group: [{select: first, group: [variable: publisher-place,
  message: term.place-unknown]}, variable: publisher]}` against a
  reference with neither `publisher-place` nor `publisher` — structurally,
  the nested group is not term-only (it contains a `Variable` child), so
  the outer `select: all` group saw "meaningful content" and rendered the
  bracketed fallback alone, even though the actual winning candidate
  *was* the term-only message and the sibling `publisher` was empty too.
  A real CSL `<group>` suppresses in exactly this case (a literal
  `<text value="...">` never counts as a rendered variable). Fixed with
  `Renderer::effective_term_only_component`
  (`processor/rendering/grouped/core.rs`), which mirrors `select:
  first`'s own winner-selection against a scratch tracker clone and
  judges the winner alone; `render_group_child_values`'s meaningful-content
  check now calls it instead of `is_term_only_component` directly.
  Regression tests: `group_select_first::select_first_term_only_fallback_does_not_make_an_enclosing_select_all_group_meaningful`
  and its complement `..._still_joins_when_a_sibling_has_real_content`,
  both verified to fail under the pre-fix structural check.
- **Blocking requirement, not a deferrable limitation:** `sorting.rs`'s
  date/contributor finders (`first_date_component_ref`/
  `first_contributor_component_ref`) check `render_when` before descending
  into a group, but have no notion of `select`. Left as-is, a `select:
  first` group's sort key would come from the structurally first
  date/contributor candidate, which may not be the candidate that actually
  renders — a reference could sort as if its first candidate applied while
  rendering its second. `render-when`'s existing imprecision here (see the
  other structural consumers above, none of which check `render_when`
  either) is tolerable because `render-when` is an input condition;
  `select: first`'s entire contract is "use the candidate that actually
  produces output," so a sort key that can silently disagree with the
  rendered value undermines the primitive's own promise, not just a
  cosmetic edge case. These two functions need to resolve a `select:
  first` group's winning candidate the same way rendering does (tried in
  order, first that renders wins) before this spec is correct — see
  Acceptance Criteria. **Update (2026-09-08):** "isolated state" doesn't
  map onto `sorting.rs` — there is no tracker to isolate here, only a
  data-presence approximation (`crate::values::select_group_children` in
  `values/mod.rs`, beside `group_condition_matches`) used to pick the
  winner without running the render pipeline. `first_contributor_component_ref`
  itself stays reference-independent (it also backs
  `*_may_have_list_primary` structural queries that ask "could any
  reference make this render," with no specific reference to resolve a
  winner against); a new reference-specific sibling,
  `first_contributor_component_ref_for_reference`, is select-aware and
  used by the two reference-specific callers
  (`primary_contributor_for_citation`/`primary_contributor_for_bibliography`).
  **Update (2026-09-08, adversarial review, before merge): the
  data-presence approximation above was itself wrong for most component
  kinds.**
  `component_would_render_for_select_first` modeled only `Group`, `Date`,
  and `Contributor`; every other kind (`Variable`, `Message`, `Title`,
  `Number`, `Term`) defaulted to `true` regardless of whether the
  reference actually had that data, and `rendering.suppress` was checked
  only for `Group`. Concretely: `select: first` on `[variable: doi, date:
  issued]` for a reference with no DOI — the real renderer falls through
  to `date: issued`, but the structural winner-picker chose `doi` (since
  `Variable` always reported "would render") and found no date at all.
  Fixed by adding a `TemplateComponent::Variable` arm backed by a new
  `simple_variable_would_render` (`values/mod.rs`), a hand-synced,
  render-context-free shadow of `values/variable.rs`'s
  `resolve_variable_value` covering every `SimpleVariable` kind except
  `Locator` (still an accepted `true` default — needs
  `options.locator_raw`, unavailable here); and by hoisting the
  `rendering.suppress` check to apply unconditionally, before the
  per-kind match. New tests in `sorting.rs`'s `select_first_winner_resolution`
  module, one verified to fail without the fix.
- **Found by the same review:** `find_template_title_node`
  (`values/contributor/substitute.rs`, used by `title-rendering:
  from-template` substitution) recursed into a `Group`'s children
  structurally, ignoring `select`. A `select: first` group's losing first
  candidate could supply substitution formatting (quote/emph) for a title
  that a later candidate actually renders. This was a known gap from this
  spec's own implementation, tracked as `csl26-msqg` and deliberately out
  of scope at the time; fixed now, once `select_group_children` above was
  itself correct, by swapping the raw `&group.group` recursion for
  `select_group_children(group, reference)` — one line. New test in
  `values/tests.rs`, verified to fail without the fix.
- Nesting a `select: first` group inside another group (of either mode),
  or vice versa, needs no special handling — it's ordinary `Group`
  recursion through the same per-component dispatch.
- **Validation is semantic, not schema-expressible.** The two `select:
  first` rules (reject fewer than two children; reject combined with
  `delimiter`) are cross-field constraints, the same category
  `RENDER_WHEN_CONTRACT.md`'s validation already isn't representable in
  the generated JSON Schema. A schema-only editor or GUI can produce a
  `select: first` group that looks valid and only fails when the style is
  loaded. Any editor built against the schema needs to also run the real
  style validator — schema validation alone is not sufficient to catch
  these two rules.

## Acceptance Criteria

- [x] `csl26-2hr4` resolved (blocking — see Implementation Notes).
- [x] `TemplateGroup.select: TemplateGroupSelect` added (`All`/`First`,
      `All` the serde default), validated (rejects `select: first` with
      fewer than two children, or combined with `delimiter`).
- [x] Evaluation implemented as a mode branch in
      `render_group_component_with_format`/`render_group_child_values`;
      candidates evaluated against a cloned tracker, only the winner's
      delta merged back.
- [x] Behavior tests: first/only/no child renders; a term-only message as
      a direct child renders correctly with no group-wrapping needed; a
      losing candidate's variable-once state doesn't leak to later
      components; a winning candidate containing a suppressed nested
      group doesn't leak that group's tracker state (the forcing case for
      the `csl26-2hr4` prerequisite); `select: first` nested inside
      `select: all` and vice versa; `render_when` on the same group
      combines correctly with `select`.
      (`crates/citum-engine/src/processor/rendering/tests.rs::group_select_first`,
      8 tests.)
- [x] `sorting.rs`'s `first_date_component_ref`/`first_contributor_component_ref`
      resolve a `select: first` group's winning candidate using the same
      selection semantics as rendering (tried in order, isolated state),
      not structural-first lookup — with a test proving sort order matches
      rendered content when the first candidate is empty and a later one
      wins. (`crates/citum-engine/src/sorting.rs::select_first_winner_resolution`,
      3 tests, verified to fail under the pre-fix structural-first lookup;
      see the Implementation Notes update on `first_contributor_component_ref`'s
      split.)
- [x] `just schema-gen` run, schema docs updated.
- [x] T&F-CSE's publisher-place fallback migrated as a worked example,
      `report-core.js` diff shows 0 regressions. Chicago's volume-title
      position and the NLM DOI case are explicitly not targets for this
      spec (see "Relationship to existing mechanisms"). The shipped CSL
      (`taylor-and-francis-council-of-science-editors-author-date.csl:77-86`)
      uses a literal `<text value="[place unknown]"/>`; migrated as
      `message: term.place-unknown` (new locale term, English text copied
      verbatim from the shipped CSL — not an invented string) with
      `wrap: { punctuation: brackets }` supplying the brackets, rather
      than baking `[...]` into the term text (Citum's existing semantic
      wrap mechanism, consistent with other bracketed content in the
      corpus). Both occurrences in
      `taylor-and-francis-council-of-science-editors-author-date-core.yaml`
      (`chapter` type-variant and the base `template:`) updated. Verified
      directly via `citum render refs`: publisher-place absent + publisher
      present renders `[place unknown]: Acme Press`; both present renders
      the real place with no bracket; both absent renders nothing (see the
      `is_term_only_component` finding above). `csl26-wj72` closed as
      done-in-PR2 rather than deferred.
- [x] Status promoted to Active in the implementation commit.

## Changelog

- v1.1 (2026-09-08): Implemented. Status promoted to Active. Corrected
  the "two competing suppression rules never arise" claim: they arise at
  the `select: first`-nested-in-`select: all` boundary, fixed via
  `Renderer::effective_term_only_component`. T&F-CSE worked example
  migrated (not deferred). An adversarial review of this same
  not-yet-merged implementation then found two more real bugs, folded in
  before merge: `component_would_render_for_select_first`'s data-presence
  approximation defaulted every unmodeled kind (`Variable`, `Message`,
  `Title`, `Number`, `Term`) to "would render" regardless of actual data,
  and only checked `rendering.suppress` for `Group` — both fixed (see the
  sorting.rs Implementation Notes update); `find_template_title_node`
  ignored `select` when recursing into groups, letting a losing
  candidate's formatting leak into `title-rendering: from-template`
  substitution — fixed by routing through `select_group_children`,
  closing `csl26-msqg`.
- v1.0 (2026-09-08): Initial draft.
