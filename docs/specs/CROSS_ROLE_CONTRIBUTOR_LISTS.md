# Cross-Role Contributor Lists Specification

**Status:** Draft
**Version:** 0.1
**Date:** 2026-07-13
**Supersedes:** None
**Related:** bean `csl26-7ip9`,
[CSL schema#442](https://github.com/citation-style-language/schema/issues/442),
[`SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md`](./SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md),
[`ROLE_LABEL_DEFAULTS.md`](./ROLE_LABEL_DEFAULTS.md),
[`ROLE_SUBSTITUTE_FALLBACK.md`](./ROLE_SUBSTITUTE_FALLBACK.md),
[`CONTRIBUTOR_PHRASE_MESSAGES.md`](./CONTRIBUTOR_PHRASE_MESSAGES.md)

## Purpose

Define how a style renders contributors from **multiple roles as one merged name
list** — interleaved entries with per-name or per-group role labels, and a single
combined-label entry when the same person holds more than one role. This is the
generalization CSL 1.0 never had: its hardcoded `editortranslator` virtual
variable covers exactly one role pairing, fires only on whole-list identity, and
cannot express per-name labels. The motivating requirements come from APA 7
multimedia references, which need all three behaviors at once:

```
Kogen, J. (Writer), Wolodarsky, W. (Writer), & Kirkland, M. (Director). (1992).
Whedon, J. (Writer & Director). (2003).
```

## Scope

In scope:

- Template schema: `contributor:` accepting an ordered role list, plus a `merge:`
  configuration block (ordering, label modes, per-role overrides, same-person
  combination).
- Rendering semantics: entry ordering, individual/collective/none labeling,
  same-person detection and combined-label resolution.
- Locale model for combined-role terms and the role-term connector.
- An options-level same-person role-suppression rule for elision across
  components.
- Migration mapping from CSL 1.0 `<names variable="editor translator">` and the
  `editortranslator` term.

Out of scope:

- Repositioning contributors into the author slot (e.g. MLA's editor-translator
  as primary entry) — that is substitution, covered by
  [`ROLE_SUBSTITUTE_FALLBACK.md`](./ROLE_SUBSTITUTE_FALLBACK.md).
- `CitationCollapse` — the citation-cluster collapse mechanism is unrelated;
  this spec deliberately avoids the word "collapse" in schema keys.
- New role-taxonomy values (e.g. a dedicated `executive-producer` sub-role) —
  bean `csl26-013w`. Examples here use existing roles only.
- Authoring combined-role terms for every role pair and locale; this spec ships
  en-US exemplars only.
- Locale-owned phrase realization around the merged list —
  [`CONTRIBUTOR_PHRASE_MESSAGES.md`](./CONTRIBUTOR_PHRASE_MESSAGES.md) remains
  the owner of cross-argument phrase grammar.

## Design

### Requirements

Style-guide research (bean `csl26-7ip9`) decomposes cross-role rendering into
three sub-problems that recur across APA 7, MLA 9, and Chicago 17 for books,
translations, film, television, and music:

| Sub-problem | Example |
|---|---|
| **A** — role-labeled interleaving | APA episode: `Kogen, J. (Writer), Wolodarsky, W. (Writer), & Kirkland, M. (Director)` |
| **B** — individual vs. collective labels | APA labels each writer individually but producers collectively (`(Producers)`, pluralized) |
| **C** — same person in multiple roles | `Whedon, J. (Writer & Director)`; `(J. Strachey, Ed. & Trans.)`; MLA `edited and translated by` |

The design constraints: identity detection is per-person, not whole-list; the
role-term connector is independent of the name-list conjunction; label placement
(before/after) and individual/collective mode are style declarations, per role;
collective labels pluralize; combined labels are declared locale terms, never
mechanically composed when an authored term exists; and full elision of a
secondary rendering (APA song: omit `[Recorded by …]` when songwriter and
performer coincide) must be expressible.

### Schema surface

`TemplateContributor.contributor` accepts a single role (unchanged) or an
ordered list of roles. A `merge:` block configures the merged list and is valid
**only** in list form; the singular `label:` field is valid **only** in
single-role form (per-role labels move under `merge.roles`). Both misuses are
schema-validation errors.

```yaml
- contributor: [writer, director]
  form: long
  merge:
    order: document            # document | role          (default: document)
    labels: individual         # individual | collective | none   (default: individual)
    roles:                     # optional per-role overrides
      writer:
        label:                 # full RoleLabel: term, form, placement,
          term: writer         # text-case, prefix, suffix
          form: long
          text-case: capitalize-first
          prefix: " ("
          suffix: ")"
      producer:
        labels: collective
    combine-same-person: true  # default: true
    role-conjunction: " & "    # optional: verbatim connector for the composed
                               # combined-label fallback (default: locale term)
  delimiter: ", "              # existing name-list options apply to the
  and: symbol                  # whole merged list
  shorten: { min: 21, use-first: 19 }
```

All existing whole-list options — `form`, `name-order`, `name-form`,
`delimiter`, `sort-separator`, `shorten`, `and`, rendering affixes, `links`,
`gender` — apply to the merged list exactly as they do to a single-role list.

### Entry ordering (sub-problem A)

The merged list is built from the reference's unified `contributors` vec
(`ContributorEntry`, `crates/citum-schema-data/src/reference/contributor.rs`),
filtered to the declared roles.

- `order: document` (default) — entries keep the order of the reference's
  contributors vec. This reproduces source-credit order (APA episode: writers
  first, then director, as credited).
- `order: role` — entries are grouped by role, groups ordered by the declared
  role-list order (MLA topology: `directed by Jane Doe, written by John Smith`).

The name-list conjunction (`and`), delimiter, and et-al shortening apply across
the full merged list regardless of ordering mode. Shortening runs **after**
same-person combination: et-al thresholds count distinct rendered entries, not
raw credits, so a reference with 22 credits where three pairs combine is a
19-entry list for threshold purposes.

`order` and `labels` are orthogonal axes; every combination is valid. In
particular, `order: role` with `labels: individual` (labeling individuals
within a role group, as MLA sometimes requires) is supported, not merely the
pairings the examples show.

### Role labels in merged lists (sub-problem B)

`merge.labels` sets the default label mode; `merge.roles.<role>.labels`
overrides it per role (APA's asymmetry: individual for writer/director,
collective for producers).

- `individual` — each entry carries its own label. Per-entry label resolution
  follows the existing chain from
  [`ROLE_LABEL_DEFAULTS.md`](./ROLE_LABEL_DEFAULTS.md), with the entry's role:
  explicit `merge.roles.<role>.label` → configured role presets → `role.defaults`
  bundle. Placement, form, case, and affixes reuse the existing `RoleLabel`
  machinery; labels are always singular in this mode.
- `collective` — one label per run, resolved by the same chain, pluralized
  when the run has two or more names. A **run** is a maximal contiguous
  sequence of same-role entries in the effective rendered order. Under
  `order: role` each role forms exactly one run; under `order: document`
  interleaved credits (writer, director, writer) yield multiple runs for a
  role — two writer runs here, each labeled per its own count. That is
  intended: preserving credit order is the point of `document`; a style that
  wants one coalesced label per role declares `order: role`.
- `none` — no label for that role's entries.

### Same-person combination (sub-problem C)

When `combine-same-person: true` (default), entries in different declared roles
that resolve to the same person render **once**, at the position of the
person's first occurrence in the effective ordering, with a combined role
label. Detection is per-person: contributors who appear in only one role are
unaffected, fixing the whole-list-only limitation of CSL's `editortranslator`.

Two entries denote the same person when their names are equal after Unicode NFC
normalization and whitespace trimming, comparing structured names field-wise
(family, given, dropping particle, non-dropping particle, suffix — all must
match exactly) and literal names as whole strings. A structured name and a
literal name never denote the same person, even when their display forms
coincide — no display-form normalization is attempted, since it invites false
positives; mixed-form duplicates are an input-data defect to fix at the
source. There is no fuzzy matching: `Smith, J.` and `Smith, John` are
distinct. The data model has no person identifier, so literal equality is the
deterministic contract (the same rule citeproc-js applies to
`editortranslator`).

`combine-same-person: false` renders the person once per declared role: each
occurrence keeps its own position in the effective ordering and carries that
role's label; no occurrence is dropped.

### Combined-label resolution

The combined label for a person holding roles *r₁, r₂, …* (in declared
role-list order) resolves as:

1. **Authored combined term** — a locale roles entry keyed by the hyphen-joined
   role names, e.g. `writer-director`, `editor-translator`. The en-US locale
   already ships `editor-translator` (`ed. & trans.`, `edited & translated
   by`, …) with full form and plural coverage; `writer-director` is added with
   this feature. Authored terms win because combined labels are not reliably
   composable — abbreviation level, capitalization, and word order may differ
   from the constituent terms.
2. **Composed fallback** — when no authored term exists, the constituent role
   terms (each resolved at the label form in effect) are joined with a
   connector inserted verbatim: `merge.role-conjunction` when set on the
   component, otherwise the locale's `role-conjunction` term. The term is a
   new general locale term (en-US: `" & "`, spacing included), deliberately
   independent of the name-list conjunction — the `&` in `(Ed. & Trans.)`
   joins role terms, not names. Verbatim insertion keeps spacing explicit:
   styles that need `/`, `" and "`, or plain concatenation (`""`) declare
   exactly that.

Form selection (short/long/verb/…) and placement for the combined label follow
the label configuration resolved for the first role in declared order **on
this component, in the current render context** — the same per-entry chain
used for individual labels (`merge.roles.<r₁>.label` → configured role
presets → context-gated `role.defaults` bundle, per
[`ROLE_LABEL_DEFAULTS.md`](./ROLE_LABEL_DEFAULTS.md)). An author-position
component and a post-title parenthetical component each carry their own label
configuration, so the combined label's shape always follows the component
being rendered, never a global per-role setting.

Plural forms apply when two or more persons share the identical role
combination. Authored combined-role terms are therefore a locale-content
obligation: every pair term must ship the full form coverage of an ordinary
role term, plurals included (`writers & directors`); the composed fallback
pluralizes each constituent term when the plural applies. The unhyphenated
`editortranslator` locale key is retained as a CSL-compatibility alias
resolved identically to `editor-translator`.

### Same-person role suppression

APA's music rule — omit the `[Recorded by …]` descriptor entirely when
songwriter and performer are the same person — is a zero-output case that no
label mechanism can express, because the affected text lives in a different
template component. It is nevertheless not a template decision: which role is
redundant when two roles coincide is a property of the style, not of any one
template position. It is therefore declared in options, not in the template
language (which this spec leaves untouched — in particular, `render-when`
grows no new condition kinds):

```yaml
contributors:
  suppress:
    - role: performer
      when-identical-to: composer
```

A rule fires **if and only if** both roles resolve to non-empty person sets
that are equal under the same-person rule above. While it fires, the
suppressed role (`role:`) renders empty in every template component, citation
and bibliography alike; existing group empty-collapse then removes dependent
descriptors such as the `[Recorded by …]` wrapper, whose group contains the
performer component.

Equality means identity, not overlap: partially overlapping sets do not fire
the rule (two composers, one of whom is also the performer, keep the
descriptor — the performer credit genuinely differs from the composer
credit), and an absent role never fires it — an empty performer already
renders nothing.

Suppression applies before merged-list assembly: a suppressed role
contributes no entries to a merged component. A style that wants a combined
label instead of suppression uses `combine-same-person` and declares no
`suppress` rule for the pair.

### Interactions

- **Substitution** — a merged component whose declared roles are all empty
  renders nothing; if `author` is among the declared roles, the author
  substitute chain applies exactly as for an empty single-role author
  component. Role-substitute suppression
  ([`ROLE_SUBSTITUTE_FALLBACK.md`](./ROLE_SUBSTITUTE_FALLBACK.md)) applies
  per declared role: a role consumed by substitution elsewhere is excluded
  from the merged list.
- **Suppress-author** — `options.suppress_author` suppresses a merged component
  that declares `author`, mirroring single-role behavior.
- **Sorting** — the component's sort key is the merged list in rendered order,
  after same-person combination, same as any name list.
- **Disambiguation** — names in a merged list participate in name-based
  disambiguation identically to single-role lists.
- **Integral citations** — the subsequent-form rewrite applies to the merged
  component unchanged.

### Migration

`citum-migrate` maps CSL 1.0 constructs:

| CSL 1.0 | Citum |
|---|---|
| `<names variable="editor translator">` | `contributor: [editor, translator]` with `combine-same-person: true` (citeproc-js merges on identical lists; per-person detection is expected to match or improve on it for real-world data — an assumption, not a guarantee: output can differ when distinct same-family contributors lack given names in the data) |
| `editortranslator` term references | authored `editor-translator` locale term (the unhyphenated alias also resolves) |
| Sequential multi-variable `<names>` with per-variable labels | `order: role` with per-role labels under `merge.roles` |

## Implementation Notes

- The single-role path in `crates/citum-engine/src/values/contributor/mod.rs`
  (`values()`) stays as-is; list form dispatches to a new merged-list renderer
  that builds `(FlatName, role)` entries from the unified contributors vec and
  reuses the existing name formatting and `labels.rs::resolve_role_labels`
  per entry/run. `resolve_role_labels` needs the role as a parameter rather
  than reading it from the component.
- Serde: `contributor:` becomes an untagged single-or-sequence; `merge:` is a
  new optional struct on `TemplateContributor`. Validation (list ⇔ `merge`,
  list ⇒ no singular `label:`) belongs in the existing style-validation layer,
  not deserialization.
- The `contributors.suppress` check slots into the same engine path as
  substitute-driven role suppression
  (`substitute::is_role_suppressed_by_substitute`,
  `crates/citum-engine/src/values/contributor/`): both answer "does this role
  render nothing for this reference?" before name formatting begins.
- The en-US `editor-translator`/`editortranslator` role terms already exist and
  are currently unconsumed; this feature is their first consumer. New content:
  `writer-director` (with the full form coverage of an ordinary role term,
  plurals included) and the general `role-conjunction` term.
- Schema regeneration (`just schema-gen`) must accompany the implementation
  commits that touch `citum-schema-style`.
- Fixture shapes per behavior follow the test-coverage skill checklist;
  parameterized integration tests use BDD `given_…_when_…_then_…` naming.

## Acceptance Criteria

- [ ] `contributor:` accepts a single role or an ordered role list; `merge:` is
      rejected in single-role form and singular `label:` is rejected in list
      form; both forms round-trip through YAML serialization.
- [ ] APA TV-episode fixture renders writers and director interleaved in
      document order with individual labels and the name-list conjunction
      spanning all entries (sub-problems A + B).
- [ ] Same-person fixture renders one entry with a combined label
      (`(Writer & Director)`) resolved from an authored `writer-director` term,
      and falls back to `role-conjunction` composition when the term is
      absent, including a component-level `merge.role-conjunction` override.
- [ ] Partial-identity fixture: with shared and unshared contributors across
      two roles, only the shared person merges; others render per role.
- [ ] Editor-translator fixture consumes the shipped en-US `editor-translator`
      term (`Ed. & Trans.` shape with explicit text-case).
- [ ] MLA topology fixture renders `order: role` groups with collective
      verb-form labels preceding each group.
- [ ] Collective pluralization fixture: a run of two or more same-role names
      gets a pluralized collective label.
- [ ] Et-al shortening counts entries after same-person combination: a fixture
      whose raw credit count exceeds the `shorten` threshold stays unshortened
      because combination brings the rendered entry count below it.
- [ ] `contributors.suppress` fixtures cover the APA recorded-by case (rule
      fires, descriptor group collapses), the absent-role case (rule does not
      fire), and the partial-overlap case (rule does not fire).
- [ ] `citum-migrate` converts `<names variable="editor translator">` to a
      merged component and maps `editortranslator` term usage.
- [ ] Sorting and disambiguation integration tests cover a merged list in the
      author position.

## Changelog

- 2026-07-13: Initial Draft for bean `csl26-7ip9`.
