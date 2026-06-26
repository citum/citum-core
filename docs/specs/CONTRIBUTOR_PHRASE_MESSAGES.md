# Contributor Phrase Messages Specification

**Status:** Draft
**Version:** 0.1
**Date:** 2026-06-26
**Supersedes:** (none)
**Related:** bean `csl26-eh5c`, [`LOCALE_MESSAGES.md`](./LOCALE_MESSAGES.md),
[`SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md`](./SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md),
[`GENDERED_LOCALE_TERMS.md`](./GENDERED_LOCALE_TERMS.md)

## Purpose

Define the locale-authored message model for contributor phrases that combine
rendered contributor names, role labels, contributor metadata, punctuation, and
rendered container or title fragments.

This fills the gap left after PR #966 moved checked-in style phrase glue away
from rendered template `term:` components and into locale-owned
`message: pattern.*` calls. That migration solved simple compositional phrases
such as accessed dates and `in {$container}`. It did not solve phrases where a
locale needs to decide the relative order of names, role labels, counts,
gender-sensitive forms, punctuation, and the title or container fragment.

## Scope

In scope:

- The initial contributor phrase message IDs and their argument contracts.
- The boundary between lexical role-label lookup and whole-phrase realization.
- AMA-style `In:` editor/title phrases and APA-style container
  contributor/title phrases as motivating cases.
- Acceptance criteria for the later implementation PR.

Out of scope:

- Implementing the renderer, schema changes, locale files, or style migrations.
- Replacing `TemplateContributor.label.term` for ordinary role-label rendering.
- Contributor name declension or title/name case inflection.
- Rich message-result fragments beyond the existing `String` message evaluator.

## Design

Contributor phrases are phrase-realization messages, not term lookups. A style
still decides that a contributor phrase is needed and supplies rendered
arguments. The active locale decides how those arguments become a
natural-language fragment.

The initial message IDs are deliberately shape-specific:

| Message ID | Required arguments | Use |
| --- | --- | --- |
| `pattern.in-contributor-container` | `names`, `role_label`, `role`, `role_form`, `count`, optional `gender`, `container` | AMA-style `In:` phrases that combine an introducer, parent contributor names, a role label, and the rendered parent title/container fragment. |
| `pattern.container-contributor-title` | `names`, `role_label`, `role`, `role_form`, `count`, optional `gender`, `title` | APA-style phrases where container contributors and their role information must be ordered relative to a rendered parent title. |

The two IDs are not aliases. They represent different citation grammar shapes:
AMA-style phrases introduce a container with an `In:`-like construction, while
APA-style phrases place contributor information around a parent title fragment.
A later implementation may add more specialized IDs, but it should not collapse
these into a single style-family selector inside one message.

`pattern.in-contributor-container` is also a distinct message from the existing
`pattern.in-container`, not a superset of it: the former receives separated
`names`, `role_label`, and `container` arguments, while the latter receives a
single pre-joined `container` blob.

Because the locale owns punctuation inside the realized phrase, these IDs
deliberately do not take `-colon` sibling variants like
`pattern.in-container-colon`; punctuation selection moves into the message body.

### Argument Contract

The implementation PR must pass these arguments to contributor phrase messages:

| Argument | Type | Meaning |
| --- | --- | --- |
| `names` | rendered string | The contributor names rendered by the normal contributor renderer, with name order, initials, list shortening, and conjunction already applied. |
| `role_label` | rendered string | The localized lexical role label, resolved from `label.term` or the effective role preset, including form, count, and gender where available. |
| `role` | selector string | The semantic contributor role, such as `editor`, `translator`, or `container-author`. |
| `role_form` | selector string | The selected label form or preset-reduced form, such as `short`, `long`, `verb`, or a future normalized role-label form. |
| `count` | number | The number of contributors represented by `names`, for plural dispatch. |
| `gender` | selector string, optional | The contributor-derived grammatical gender selector when the existing role-label rules can derive one. Omit when no stable selector exists. |
| `container` | rendered string | For `pattern.in-contributor-container`, the rendered parent title/container fragment. |
| `title` | rendered string | For `pattern.container-contributor-title`, the rendered parent title fragment. |

`role_label` is a rendered lexical argument, not a message ID. This lets the
locale author decide where the label sits in the whole phrase without losing
the existing typed role-label machinery.

Placeholder names use `snake_case` (`{$role_label}`, `{$role_form}`). This
extends the existing single-token convention (`{$date}`, `{$container}`) and is
valid MF2 variable syntax, since the underscore is a name character.

### Why Existing Features Are Insufficient

The deprecated template-schema `term:` component is insufficient because it
renders one localized term at a fixed point in the template. It cannot receive
rendered names and titles as arguments, cannot reorder those arguments, and
cannot express punctuation that depends on the full phrase shape. Using it for
phrases forces English word order back into the style template.

`TemplateContributor.label.term` remains necessary, but it is a lexical label
mechanism. It answers "which localized role label should appear for this
contributor role, form, count, and gender?" It does not answer "where should
that label appear relative to names, an `In:` introducer, and a parent title in
this locale?"

`message: term.*` is also lexical. It lets template messages resolve terms
through the same locale path as other term lookups, including form and gender
support. It is not a whole contributor phrase model because it has no rendered
`names`, `role_label`, or `title`/`container` argument contract.

The existing `pattern.in-container` message receives a pre-rendered
`container` blob. That is useful for simple phrases such as `in {$container}`,
but it hides the internal parts from the locale. Once a style has already
rendered `editors + label + title` into one argument, the locale cannot move
the role label before the names, place the title before the contributors,
change punctuation around the role label, or choose a different phrase shape
for singular, plural, mixed-gender, or role-specific cases.

Secondary contributor role presets are insufficient for the same reason. They
choose whether a role label is absent, prefixed, or suffixed around a name list.
They do not own the larger phrase that includes a container introducer or
parent title. Presets remain the right low-level control for label rendering,
but contributor phrase messages own the cross-argument grammar.

These limits matter across diverse locales. Some locales place role labels
before names rather than after them; some require different punctuation around
an introduced container; some need count or gender selectors to choose a valid
role-label surface; and some naturally place the parent title before the
contributors. A style template that bakes in English sequence and punctuation
cannot express those differences without duplicating the style per locale.

### Locale Examples

Fills are held constant — `names` = `Smith, J.`, `container`/`title` = `The Handbook` — so
every difference in the rendered result is attributable to the locale message, not the data.
Role-label surfaces are illustrative.

`pattern.in-contributor-container` — introducer, order, and punctuation all vary by locale:

| Locale | Message | Rendered result |
| --- | --- | --- |
| en-US (introducer-first, prepositional) | `In: {$names}, {$role_label}. {$container}` | `In: Smith, J., ed. The Handbook` |
| fr-FR (French colon spacing) | `Dans : {$names}, {$role_label}. {$container}` | `Dans : Smith, J., éd. The Handbook` |
| tr-TR (postpositional — container first) | `{$container} içinde, {$role_label}: {$names}` | `The Handbook içinde, haz.: Smith, J.` |

**Number and gender act upstream.** `count` and `gender` usually select the agreeing
`role_label` / `names` surfaces *before* the message runs, so one template covers several
forms. `gender` comes from the contributor record (`ContributorGender`), never from the name.

By `count` (en-US; this row adds a second contributor to show the effect):

| `count` | Rendered result — message unchanged |
| --- | --- |
| 1 | `In: Smith, J., ed. The Handbook` |
| 2 | `In: Smith, J. and Jones, A., eds. The Handbook` |

By `gender` (es-ES, long editor label, message `En: {$names}, {$role_label}. {$container}`):

| `gender` | Rendered result — message unchanged |
| --- | --- |
| masculine | `En: Smith, J., editor. The Handbook` |
| feminine | `En: Smith, J., editora. The Handbook` |

A locale whose phrase *frame* — not just the label — changes with number or gender can branch
on `count` or `gender` inside the message. When contributors mix genders or carry none, the
agreement rules fall back to a default surface and the message omits `gender`.

`pattern.container-contributor-title` — the title may precede the contributors:

| Locale | Message | Rendered result |
| --- | --- | --- |
| en-US (contributors first) | `{$names} ({$role_label}), {$title}` | `Smith, J. (ed.), The Handbook` |
| de-DE (title first) | `{$title}, {$role_label}: {$names}` | `The Handbook, Hrsg.: Smith, J.` |
| it-IT (role as phrase, locale separator) — illustrative | `{$title} / {$names} ({$role_label})` | `The Handbook / Smith, J. (a cura di)` |

These mirror real `pattern.in-container` orderings already shipped in the embedded locales
(`en-US`/`fr-FR` prepositional vs. `tr-TR`/`eu-ES` postpositional). The `count`, `gender`, and
it-IT rows are illustrative — Italian is not yet an embedded locale and the `count` row varies
the contributor fill — but they show the same point: number, gender, and multi-word role-label
surfaces are resolved before the message, and separators are locale-chosen. They are not
normative output for any specific locale; they show why the locale must receive separate
rendered arguments — `names`, `role_label`, and the title/container fragment — instead of one
prejoined string, so it can own both order and punctuation.

## Implementation Notes

The implementation should extend the existing `TemplateMessage` path rather
than add a second message system. The style should continue to call a
`pattern.*` message, and each argument should be rendered by normal template
components before message evaluation.

The current evaluator returns `String`, so rendered rich-text spans inside
arguments are still flattened at the message boundary. That limitation already
exists for `pattern.in-container`; this spec does not expand it. A later
fragment-output design can preserve rich placeholder fragments without changing
the argument names defined here.

When deriving `gender`, reuse the existing contributor-role label agreement
rules from `GENDERED_LOCALE_TERMS.md`. Do not infer gender from names. If mixed
or absent contributor gender prevents a stable selector, omit `gender` and let
the locale message use wildcard/default branches.

## Acceptance Criteria

- [ ] The later implementation supports `pattern.in-contributor-container`
      with `names`, `role_label`, `role`, `role_form`, `count`, optional
      `gender`, and `container` arguments.
- [ ] The later implementation supports `pattern.container-contributor-title`
      with `names`, `role_label`, `role`, `role_form`, `count`, optional
      `gender`, and `title` arguments.
- [ ] `TemplateContributor.label.term` continues to resolve lexical role
      labels and is not linted as deprecated template `term:`.
- [ ] AMA coverage includes a chapter or conference-paper case where editor
      names, an editor label, and a parent title are passed separately to
      `pattern.in-contributor-container`.
- [ ] APA coverage includes a container-contributor/title case passed to
      `pattern.container-contributor-title`.
- [ ] At least one reordered-locale fixture proves the locale can move the
      rendered title/container fragment relative to names and the role label.
- [ ] Tests cover singular and plural contributor counts; gender is covered
      where the existing contributor gender data can derive a stable selector.

## Changelog

- 2026-06-26: Initial Draft for bean `csl26-eh5c`.
