# Authoring Locales

This guide tells you when and how to add MessageFormat 2 (MF2) entries to a
Citum locale file in `locales/`. It is the practical companion to
[`docs/specs/LOCALE_MESSAGES.md`](../specs/LOCALE_MESSAGES.md).

## Status

- MF2 evaluator: live in `crates/citum-schema-style/src/locale/message.rs`.
- Engine call sites: live. `resolved_locator_term` and `resolved_role_term`
  are wired in `crates/citum-engine/src/values/{locator,number,contributor/labels}.rs`
  and consult the `messages:` map first, falling back to the legacy `terms:` /
  `roles:` / `locators:` maps.
- Gendered term values: live through `MaybeGendered<T>` in the legacy locale
  maps. This is separate from MF2 selector support.
- Coverage as of writing: `en-US`, `de-DE`, `fr-FR`, `tr-TR`, `es-ES` carry
  v2 `messages:` blocks. Adding new locales should follow the same shape.

If a `messages:` block is absent or empty, the engine silently uses the legacy
maps. There is no per-locale fallback alarm — author intent is signalled by
`evaluation.message-syntax: mf2` in the locale header.

## When to add a `messages:` entry

Author a v2 `messages:` entry whenever the rendered text depends on a
**parameter** that the legacy term tables can't carry compositionally:

| Pattern | Example | Use MF2? |
|---------|---------|----------|
| Plural-dependent abbreviation | `p.` vs `pp.` | **Yes** — `.match {$count :plural}` |
| Plural-dependent long form | `chapter` vs `chapters` | **Yes** |
| Plural-dependent verb | "with guest" vs "with guests" | **Yes** |
| Compositional pattern | `{$start}–{$end}` for page ranges | **Yes** |
| URL-bearing message | "retrieved from {$url}" | **Yes** |
| Static label | `term.and: "and"` | Optional — works either way |
| Gender-dependent label | Spanish `editor / editora`, French `éditeur / éditrice` | **Not yet** — see "Gender" below |

The rule of thumb: if the message body would contain `{` (variable
substitution or `.match` block), MF2 is the correct home. If it's a bare
string, both paths render identically; MF2 is preferred when authoring a new
locale for consistency with the v2 schema.

## Required block when `message-syntax: mf2`

A v2-aware locale should ship the following four blocks at the bottom of the
file, after `terms:`, `roles:`, and `locators:`:

```yaml
messages:
  # plural-dispatched and parameterized strings — see catalog below
date-formats:
  numeric-short: "..."
  textual-long: "..."
  textual-full: "..."
  bib-default: "..."
  year-only: "yyyy"
  iso: "yyyy-MM-dd"
grammar-options:
  punctuation-in-quote: false
  open-quote: "..."
  close-quote: "..."
  serial-comma: false
  page-range-delimiter: "..."
legacy-term-aliases:
  # bridge old engine keys to new message IDs
```

Use `locales/en-US.yaml`, `fr-FR.yaml`, or `de-DE.yaml` as references for
gender-invariant messages. `es-ES.yaml` is the current concrete example of a
locale that carries MF2 messages while keeping gendered role labels in `roles:`
until MF2 can dispatch on both `$gender` and `$count` in one message. The same
limitation applies to other gendered locales, including French and Arabic.

## Message ID catalog (frequently used)

| ID | Variables | Notes |
|----|-----------|-------|
| `term.page-label`, `term.page-label-long` | `$count` | `p./pp.` and `page/pages` equivalents |
| `term.chapter-label`, `term.chapter-label-long` | `$count` | |
| `term.volume-label`, `term.volume-label-long` | `$count` | |
| `term.section-label`, `term.section-label-long` | `$count` | |
| `term.figure-label` | `$count` | |
| `term.note-label`, `term.note-label-long` | `$count` | |
| `term.and`, `term.and-symbol`, `term.et-al`, `term.and-others` | none | Conjunctions |
| `term.accessed`, `term.retrieved`, `term.no-date`, `term.no-date-long`, `term.forthcoming`, `term.circa`, `term.circa-long` | none | Date and access labels |
| `role.editor.label`, `role.editor.label-long`, `role.editor.verb` | `$count` (label only today) | Avoid for gender-aware locales until multi-selector MF2 lands |
| `role.translator.label`, `role.translator.label-long`, `role.translator.verb` | `$count` (label only today) | Avoid for gender-aware locales until multi-selector MF2 lands |
| `role.guest.label`, `role.guest.label-long`, `role.guest.verb` | `$count` | |
| `pattern.page-range` | `$start`, `$end` | Spec'd; not yet consumed by the engine |
| `pattern.retrieved-from`, `pattern.available-at` | `$url` | Spec'd; not yet consumed by the engine |
| `date.open-ended` | none | "present" / "heute" / "presente" |

> **Note on `pattern.*` messages.** No engine call site currently passes
> `$start` / `$end` / `$url` into the evaluator. These IDs are reserved by
> the spec and locales already author them, but consumers will land in a
> separate change. Author them in new locales for forward-compatibility,
> but don't expect them to render today.

The `legacy-term-aliases:` map bridges single-word legacy keys (e.g. `page`,
`et_al`, `editor`) to message IDs so styles authored against the v1 vocabulary
keep rendering. Mirror the en-US shape.

## Gender — interim limitation

`MaybeGendered<T>` is already live for locale term maps. The `roles:`,
`locators:`, and `terms:` fallback paths can store and resolve gendered values
for locales that need them, such as Spanish role nouns, French role nouns, and
Arabic gendered ordinals, using the requested gender from contributor data or
an explicit template override.

The MF2 path is not equivalent yet. `resolved_role_term` and
`resolved_locator_term` currently pass `$count` into MF2 evaluation, but they do
not pass `$gender`. The custom evaluator also supports only one selector per
`.match`, so a full `$gender` x `$count` role-label matrix cannot be authored
reliably today.

For this branch:

- Locale files for languages with role-label gender variants keep those labels
  in `roles:`. Do not add `role.editor.label`,
  `role.editor.label-long`, `role.translator.label`, or
  `role.translator.label-long` MF2 entries for those locales yet.
- This is an implementation gap, not the desired final architecture. MF2 should
  become the home for gender x plural role labels after multi-selector support
  lands.
- Gender-invariant role messages are safe in `messages:`.
- Gender-invariant locator and general terms are safe in `messages:`.

The follow-up is to pass gender into `MessageArgs`, support multi-selector
`.match`, and then migrate gendered role labels from `roles:` to MF2 with tests.

## Runtime locale selection

Styles can declare a default locale, but that is not the only intended source
of locale choice. `citum render refs --locale <locale-id>` is a user-facing
per-invocation override for choosing any available locale at render time; the
verification command below uses it as a smoke test because it exercises that
same runtime selection path. Persistent user default locale configuration is
tracked separately in the user style and locale store work (`csl26-erwz`).

## MF2 syntax cheat sheet

```yaml
# Static string
term.and: "and"

# Variable substitution
pattern.retrieved-from: "retrieved from {$url}"

# Plural dispatch (one / wildcard only)
term.page-label: |
  .match {$count :plural}
  when one {p.}
  when * {pp.}

# Select dispatch (arbitrary keys, one selector only today)
some.gendered: |
  .match {$gender :select}
  when masculine {él}
  when feminine {ella}
  when * {elle}
```

The current evaluator supports only `one` and `*` plural categories. Full
CLDR categories (`zero`, `two`, `few`, `many`) and multi-selector `.match`
blocks require follow-up evaluator work. ICU4X MF2 support is tracked, but not
treated as available in this branch.

## Verification

After editing a locale, run:

```bash
# 1. Schema check — ensures parse + evaluator setup is sound
cargo nextest run -p citum-engine -E 'test(locale) + test(spanish) + test(es_es) + test(german) + test(french)'

# 2. Portfolio gate — guards against accidental breakage of style fidelity
node scripts/report-core.js > /tmp/r.json && \
  node scripts/check-core-quality.js \
  --report /tmp/r.json \
  --baseline scripts/report-data/core-quality-baseline.json

# 3. Smoke test against a real style
cargo run --bin citum -- render refs \
  -b tests/fixtures/references-expanded.json \
  -s styles/apa-7th.yaml --locale <your-locale>
```

A future `citum locale lint <file>` (spec §8) will short-circuit the first
two — not yet shipped.

## Related

- Spec: [docs/specs/LOCALE_MESSAGES.md](../specs/LOCALE_MESSAGES.md)
- Gender model: [docs/specs/GENDERED_LOCALE_TERMS.md](../specs/GENDERED_LOCALE_TERMS.md)
- ICU4X migration: bean `csl26-qrpo`
- Gender-aware MF2 role labels: follow-up bean for multi-selector `.match`
