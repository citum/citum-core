# Sentence-Initial Labels Specification

**Status:** Active
**Version:** 0.6
**Date:** 2026-04-11
**Supersedes:** None
**Related:** `csl26-twx1`, `docs/specs/SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md`, `docs/specs/NOTE_START_REPEATED_NOTE_POLICY.md`, `docs/specs/TITLE_TEXT_CASE.md`

## Purpose
Define how Citum should model sentence-initial capitalization for localized
labels and markers that are normally authored in lowercase or sentence case.
The immediate trigger is contributor verb labels such as `edited by` appearing
after a period in bibliography output, but the design target is broader:
Citum needs a coherent render-context model for sentence-initial labels without
turning capitalization into a blanket string mutation rule.

## Scope
In scope:
- localized labels and markers whose rendered capitalization changes when they
  become sentence-initial
- contributor verb labels and verb-prefix presets as the motivating example
- note-start markers and other locale-backed labels that can shift between
  mid-sentence and sentence-initial positions
- sentence-case label behavior as the baseline assumption for this spec
- separation of locale-owned lexical content from render-context-owned casing
- the processor-owned render-context contract for sentence-initial behavior

Out of scope:
- shipping rendering behavior beyond the first sentence-initial implementation
  wave activated with this spec revision
- broad title/text-case semantics already covered by
  `docs/specs/TITLE_TEXT_CASE.md`
- renaming or activating any new public schema/API field in this draft
- non-localized arbitrary prose rewriting
- full sentence-boundary NLP or punctuation inference beyond Citum's rendering
  model

## Design
### Problem Statement
Current repo evidence shows a real output gap: localized contributor verb labels
such as `edited by` remain lowercase even when the rendered component becomes
sentence-initial after punctuation such as `. `. This is visible in
bibliography-style output where a sentence boundary is created by prior
component affixes, but the contributor label is still resolved from locale data
and emitted unchanged.

This failure is not unique to contributor labels. Any localized label or marker
that may appear mid-sentence in one style path and sentence-initial in another
needs context-sensitive capitalization. The engine therefore needs a principled
way to distinguish lexical content from sentence-initial render context.

This specification assumes sentence-case locale/style labels as the baseline.
Title-like casing remains governed by `docs/specs/TITLE_TEXT_CASE.md`.
Sentence-initial capitalization is orthogonal to title/text-case semantics: it
controls whether a label begins with an initial capital because of render
context, not whether the label belongs to a title-case family.

### Ownership Boundaries
Lexical content remains locale- and style-owned. Locale terms such as
`edited by`, `translated by`, or note-style markers should continue to be
authored in their normal form rather than duplicated in both lowercase and
capitalized variants.

Sentence-initial capitalization is a render-context concern. It should not be
treated as a permanent mutation of the underlying term, nor as a global rule
that rewrites any lowercase string the processor encounters.

This distinction is normative:

1. Locale/style data owns wording.
2. Rendering context owns whether a term is sentence-initial.
3. The engine must preserve lowercase or sentence-case forms when the same term
   appears mid-sentence.

### Sentence-Initial Contract
For this specification, `sentence-initial` means a boolean render-context signal
attached to a render node or equivalent processed output unit. The signal is
set by the template/render pipeline from known Citum context sources rather than
by raw punctuation scanning over flat rendered strings.

Initial context sources may include:

1. explicit note-start context already recognized by the processor
2. explicit separator or affix semantics that the rendering pipeline treats as
   sentence boundaries
3. future structured markup or render-node metadata that encodes a sentence
   break directly

This is a normative boundary:

1. raw punctuation alone is not the contract
2. sentence-initial status must come from rendering context, not post-hoc text
   inspection
3. style families may later refine which separator classes count as sentence
   boundaries, but that refinement must flow through the render-context signal
   rather than a generic string rewrite pass

### Language and Script Considerations
Sentence-initial context is a contextual flag, not a casing transform. Eligible
render nodes may carry `sentence-initial = true` regardless of script or
writing direction, including LTR, RTL, and bidirectional text.

Visible change is orthography-dependent rather than guaranteed. The existence of
sentence-initial context does not by itself imply any letter-case change.

This is a normative boundary:

1. the processor propagates sentence-initial context independently of script
   features
2. locale/style logic decides whether sentence-initial context maps to
   capitalization, label-variant choice, or no visible change
3. languages or scripts without case must treat casing as a no-op rather than
   a guessed transformation
4. when Citum lacks defined language/script-specific casing behavior, it must
   prefer `as-is` behavior over guessing, consistent with
   `docs/specs/TITLE_TEXT_CASE.md` and the fallback approach already used in
   `crates/citum-engine/src/values/text_case.rs`

### Affected Content Taxonomy
The initial taxonomy for this work is:

1. Contributor verb labels and verb-prefix presets
   - examples: `edited by`, `translated by`
2. Note-start markers whose style families treat note-initial output as
   sentence-initial
   - examples already discussed in
     `docs/specs/NOTE_START_REPEATED_NOTE_POLICY.md`
3. Other localized labels or markers that can move between prose-adjacent and
   sentence-initial slots
   - examples: future locator labels, explanatory markers, or style-authored
     context labels

The taxonomy is intentionally broader than contributor rendering, but this
specification does not require every category to ship in one implementation
wave.

### Current Engine Boundary Sources
The current engine exposes only partial signals relevant to this problem:

1. Note-start casing exists as a narrow, explicit processor path for citation
   output.
2. Component rendering already has local knowledge of prefixes, suffixes, and
   affixes.
3. Contributor role-label formatting already sees component rendering and locale
   term resolution together.
4. The engine does not currently expose a first-class sentence-boundary or
   sentence-initial render-context signal across template rendering.

This means Citum can sometimes infer sentence-initial behavior locally, but it
cannot yet express sentence-initial context as a reusable rendering dimension.

### Eligible Node Classes and Propagation
Only locale-backed label or marker nodes are eligible for automatic
sentence-initial capitalization under this specification. Arbitrary
author-supplied text must never be transformed by the sentence-initial signal.

The intended long-term home for this signal is the processed render node layer,
not the locale term store and not a renderer-wide flat-string post-pass. A
future implementation may temporarily adapt local call sites such as
contributor-label formatting, but those adapters should consume a render-context
signal rather than define the normative model.

Propagation rules should follow this direction:

1. the pipeline computes `sentence-initial` before locale-backed labels are
   formatted
2. locale-resolved label nodes inherit the signal from their containing render
   node unless a style-owned rule explicitly overrides that label class
3. nested non-label user content does not become eligible merely because it is
   adjacent to an eligible label node

### Chosen Architecture
This specification adopts one architecture only: a processor-computed
`sentence-initial` flag on processed component output.

During template-to-render-node assembly, the processor computes an explicit
boolean `sentence-initial` field from known Citum context sources. Locale-backed
label renderers then consume that field when deciding how sentence-initial
context affects rendering; they do not infer sentence starts from final text or
punctuation.

For the first implementation, this field should live on the internal processed
component layer that already sits between `ComponentValues` resolution and
renderer formatting, reusing the existing `ProcValues`/`ProcTemplateComponent`
pipeline or a direct successor at the same stage. This resolves the carrier
question for the draft: the signal belongs on processed rendering data, not in
locale terms, not in authored text, and not in a renderer-wide post-pass.

Normative direction for later implementation:

1. When a locale-backed label or marker node is rendered with
   `sentence-initial = true`, rendering must consult locale/style behavior for
   that label class rather than assuming a visible capitalization change.
2. The processor must compute that field during template-to-render-node
   assembly from explicit inputs rather than by scanning rendered strings.
3. Initial explicit inputs include note-start context and processor-owned
   sentence-boundary semantics derived from affixes, separators, or future
   structured boundary metadata.
4. The processed component layer is the required internal carrier for this
   signal in the first implementation wave.
5. Arbitrary author-supplied text must never become eligible for automatic
   transformation from this flag.
6. Temporary local heuristics are acceptable only as upstream input detection
   inside the processor path; they must not become a second architectural model
   or a renderer-side fallback rule.

### Resolved Design Decisions
This draft resolves the major design choices as follows:

1. The first implementation wave should implement the sentence-initial contract
   across the in-scope locale-backed label classes covered by this spec,
   including contributor verb labels and existing note-start markers.
   Additional locale-backed marker classes may follow in later work without
   changing the architecture.
2. Style-specific exceptions remain locale/style-owned. If a style family keeps
   a note-start marker lowercase, that exception is expressed through the label
   class's rendering rule, not by suppressing the context flag itself.
3. Language/script-aware behavior belongs to the locale/style text-case layer
   that consumes `sentence-initial`, reusing the same "prefer `as-is` over
   guessing" fallback principle already used for title text-case resolution.
4. Fixture coverage for implementation must include:
   - a positive contributor-label case in a case-changing language/script
   - a negative mid-sentence case for the same label
   - a note-start case proving non-regression against current note behavior
   - a no-op language/script case where `sentence-initial = true` produces no
     visible case change
   - an RTL or bidi case proving the flag is direction-agnostic
5. Rejected alternatives are out of scope for this spec revision:
   - local heuristics as the normative model
   - renderer-wide generic capitalization

### Migration and Compatibility Notes
This specification does not itself change runtime behavior.

When implementation follows, compatibility requirements should include:

1. Mid-sentence localized labels must remain in their authored lowercase or
   sentence-case form.
2. New capitalization behavior must be limited to render paths that are
   semantically sentence-initial, not merely punctuation-adjacent.
3. Existing note-start behavior must remain consistent with the active
   note-start policy rather than being silently replaced by a generic rule.
4. Contributor-label fixes should not force early commitment to final public
   schema naming.
5. New behavior must ship with fixtures that prove both positive cases
   (capitalized sentence-initial labels) and negative cases (no capitalization
   when the render context is not sentence-initial).
6. New behavior must include both case-changing and no-op language/script
   examples so the flag stays orthography-neutral.
7. RTL and bidi examples must demonstrate that writing direction does not alter
   the core sentence-initial contract.

### Remaining Review Questions
This revision leaves no architectural review questions open.

For future implementation guidance, the internal field name should remain
`sentence-initial` unless a later code-level constraint requires a narrowly
scoped naming adjustment.

## Implementation Notes
- Reuse existing repo evidence rather than inventing a generalized casing system
  in one step.
- Keep contributor verb labels as the motivating implementation entry point, but
  do not describe the problem as contributor-only.
- If a future schema/API field is needed, model it as an orthogonal render
  context rather than a mutation of locale terms.
- Add `sentence-initial` to the processed component layer that bridges
  `ComponentValues` output and renderer formatting, then let label-formatting
  call sites consume it.
- Treat `sentence-initial` as the intended internal field name for the first
  implementation wave.
- Reuse the repo's current language-aware fallback principle: prefer `as-is`
  over guessing when Citum lacks a defined orthographic transform.
- Treat note-start context as an explicit upstream input to this processor-owned
  flag and coordinate implementation follow-up with
  `docs/specs/NOTE_START_REPEATED_NOTE_POLICY.md` so settled note-style behavior
  is not widened accidentally.

## Acceptance Criteria
- [ ] Citum has a draft spec for sentence-initial capitalization of localized
  labels and markers.
- [ ] The spec separates locale/style-owned lexical content from
  render-context-owned capitalization.
- [ ] The spec documents the current engine boundary: note-start casing exists,
  affix heuristics exist locally, and no first-class sentence-boundary signal
  exists yet.
- [ ] The spec defines `sentence-initial` as a render-context signal rather than
  a punctuation scan over final strings.
- [ ] The spec treats sentence-initial context as script-agnostic and visible
  casing behavior as language/script-dependent.
- [ ] The spec limits automatic capitalization eligibility to locale-backed
  label or marker nodes and excludes arbitrary author-supplied text.
- [ ] The spec defines a single target architecture: a processor-computed
  `sentence-initial` flag on processed component output.
- [ ] The spec assigns the first implementation carrier to the processed
  component layer used between value resolution and renderer formatting.
- [ ] The spec requires positive and negative fixture coverage before runtime
  behavior changes ship.
- [ ] The spec requires future examples covering both case-changing and no-op
  language/script behavior, including RTL or bidi text.

## Changelog
- v0.6 (2026-04-11): Activated the spec, aligned scope with the first engine
  implementation wave, and required the processed-component sentence-initial
  contract to ship with the paired runtime behavior.
- v0.5 (2026-04-11): Removed rejected alternative options, adopted a single
  processor-owned `sentence-initial` architecture, resolved the main design
  questions, and narrowed the remaining review questions.
- v0.4 (2026-04-11): Added language/script-aware sentence-initial semantics and
  collapsed the design space to heuristics, renderer-wide auto-capitalization,
  and processor-computed render-node signaling.
- v0.3 (2026-04-11): Added the explicit `TemplateGroup` sentence-hint option as
  Option 3A and split processed render-node signaling into Option 3B.
- v0.2 (2026-04-11): Clarified the `sentence-initial` contract, preferred
  signal carrier, node eligibility, and fixture expectations.
- v0.1 (2026-04-11): Initial draft for sentence-initial localized labels and
  markers.
