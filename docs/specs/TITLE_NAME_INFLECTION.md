# Title and Proper-Noun Inflection Across Languages

**Status:** Draft
**Version:** 0.2
**Date:** 2026-05-16
**Supersedes:** (none)
**Bean:** `csl26-1b4e`
**Related:** `csl26-v6ok` (locale-authored date patterns — the easier date-component subset of the same problem), CSL upstream issue [#6369](https://github.com/citation-style-language/styles/issues/6369), `docs/specs/GENDERED_LOCALE_TERMS.md` (the closest in-repo precedent)

> **Audience.** This spec is written for engine implementers and style authors who are **not** assumed to know linguistic terminology. The "Background" section is a deliberately gentle primer; skim it if it's review.
>
> **Confidence.** Linguistic claims in this draft are marked:
> - **(verified)** — claim is supported by a published reference grammar or widely-used MT resource, cited inline.
> - **(illustrative)** — example is shaped to convey the design problem; the *form* of inflection is real, but the specific rendered string should be reviewed by a native speaker before being treated as authoritative.
> - **🚧 (needs review)** — claim has not been verified; flagged for follow-up before any implementation work.

---

## Purpose

Let Citum render citations correctly in languages where author names and work titles change their surface form depending on their grammatical role in the surrounding sentence. Today these are treated as opaque strings, so styles cannot ask for "the genitive form of this title" or "this author's name in the inessive case" even when the language requires it.

This spec is **design only**. No code changes are proposed yet. The goal is to enumerate the choices, illustrate them with concrete examples, and surface what we don't know so it can be resolved before implementation begins.

---

## Recommendation: reserve the schema, defer the renderer

The prior-art survey (below) shows essentially no reusable precedent. The user-demand survey (below) shows that the problem is real but that current CSL-stack users overwhelmingly cope by manually editing exports or accepting ungrammatical output, rather than escalating to feature requests with sustained follow-through. The implementation cost of a generic case-aware renderer is high (linguistic complexity, no native-speaker review pipeline, ongoing locale maintenance); the user-facing benefit is currently low (few sustained complaints).

This spec therefore recommends a **staged approach**:

1. **Now.** Define the input data model — the `MaybeInflected<T>` shape and the locale-declared case-set mechanism — so any future contribution has a coherent place to dock.
2. **Now.** Reserve the relevant schema surface (field naming, serialisation behaviour for `Plain` vs `Inflected` variants) so that adding a `forms:` map to a title or contributor in a future minor release is non-breaking.
3. **Defer.** Renderer implementation, MF2 case selectors, style-language plumbing, and migration tooling. Resume this when a motivated locale-specific contributor emerges with sample input data and a real style they want to ship — *not* on a speculative schedule.

The remainder of this document is written under that staged plan: schema-side design decisions (Q1, Q2, Q3) should be pressed to closure when the spec leaves Draft; rendering-side decisions (Q4, Q5) are sketched with options but explicitly held open for the locale contributor who will eventually drive them.

---

## Scope

**In scope:**

- A design framework for representing per-case variants of titles, author names, and other proper nouns in Citum's input data and locale model.
- A worked-example sketch of how a style would request a specific case form.
- An enumeration of open questions whose answers should drive — not be derived from — implementation.

**Out of scope:**

- Date-component inflection. Already solved by `csl26-v6ok` via `pattern.date-*` MF2 messages.
- Morphological *derivation* (an engine that generates inflected forms from a base form). This spec assumes inflected forms are supplied as input, consistent with Citum's "explicit over magic" principle.
- Input-side OCR / parser improvements to recover inflected forms from bibliographic databases that don't ship them.
- Locale-term gender (already handled by `MaybeGendered<T>` — see `docs/specs/GENDERED_LOCALE_TERMS.md`).
- The CSL 1.0 procedural-XML side. This spec targets the Citum-native data model; CSL-derived styles will inherit the capability via migration but the migration mechanism is a separate concern.

---

## Background: a 90-second primer on grammatical case

In English, most nouns look the same wherever they appear in a sentence:

> The **dog** is barking.  *(subject)*
> I see the **dog**.  *(object)*
> I gave the **dog** a bone.  *(indirect object)*

Only the personal pronouns inflect (*he / him / his*), and a residual genitive *'s* marks possession (*the dog's bone*). English is, by world standards, **morphologically poor**: it relies on word order and prepositions instead of case marking.

Many other languages mark these roles directly on the noun. The set of forms a noun can take is its **case system**, and each form is a **case** (German *Kasus*, Latin *casus*). The two best-known case systems in European languages:

- **Latin:** nominative, accusative, genitive, dative, ablative, vocative. The word *rosa* "rose" appears as *rosa, rosam, rosae, rosae, rosā, rosa* depending on grammatical role.
- **German:** four cases (nominative, accusative, genitive, dative). *Der Mann* "the man" appears as *der Mann, den Mann, des Mannes, dem Mann*.

Other languages go much further:

| Language | Approximate case count | Source |
|---|---|---|
| **Russian** | 6 | Standard reference grammars (e.g. Wade, *A Comprehensive Russian Grammar*) **(verified)** |
| **German** | 4 | Any reference grammar **(verified)** |
| **Finnish** | ~15 | Karlsson, *Finnish: A Comprehensive Grammar* **(verified)** |
| **Hungarian** | ~18 | Rounds, *Hungarian: An Essential Grammar* **(verified)** |
| **Basque** | ~17 | Hualde & Ortiz de Urbina, *A Grammar of Basque* (Mouton de Gruyter, 2003) **(verified)** |

For our purposes, the only things you need to remember are:

1. The same noun can have many surface forms in the same text.
2. The form is selected by the surrounding sentence, not by the noun itself.
3. The *set* of available cases is language-specific. Russian's six cases are not the same six cases that Spanish would have if Spanish had cases; Finnish's inessive ("inside something") doesn't map cleanly onto any Russian case. Languages don't merely have *more* or *fewer* cases — they carve up grammatical role space differently.

There is one additional structural axis worth knowing about, though it doesn't change the design surface much:

- **Nominative–accusative languages** (the European mainstream — Latin, German, Russian, Finnish, Hungarian) distinguish the subject of a transitive verb ("she sees the dog") from the object ("the dog sees her").
- **Ergative–absolutive languages** (including **Basque**) instead group the subject of an intransitive verb with the object of a transitive one. So the case marking on "the dog" in "the dog is barking" matches "the dog" in "she sees the dog", not the case marking on "she". **(verified)** — Hualde & Ortiz de Urbina, op. cit.

Both alignments produce names and titles that change form by context. The design we land on must not bake in nominative–accusative assumptions.

---

## The problem in citations

Bibliographic citations are not isolated noun phrases. They are embedded in running sentences (narrative citations) or in structured layouts (bibliography entries) that the style assembles in a specific language. In inflecting languages this means the author name or title routinely appears in a non-nominative case.

### Worked example 1 — Finnish narrative citation

In English, an author-date narrative citation looks like:

> According to Smith (1962), …

In Finnish, the same idea is most naturally written with the author's name in the **genitive** case, because *mukaan* "according to" takes a genitive complement. **(verified)** — Karlsson, *Finnish: A Comprehensive Grammar*.

> **Smithin** (1962) mukaan, … *(illustrative)*

`Smith → Smithin` is a regular Finnish genitive: append *-n* to the nominative stem. The case marking is required by the surrounding word *mukaan*; a Finnish reader will perceive *Smith mukaan* as ungrammatical, the way an English reader perceives *I gave he a bone*. The exact placement of the parenthesised year relative to *mukaan* is a style-guide choice; the case marking on the name is not.

A Citum-native Finnish style today cannot produce *Smithin* from input that only carries *Smith*. The style author's only options are:

1. Hardcode the genitive ending and hope every author name takes the same one (it doesn't — Finnish has multiple declension classes).
2. Bypass the renderer and put a literal *Smithin* in the input data, losing portability.
3. Accept incorrect Finnish output.

None of these is acceptable.

### Worked example 2 — Russian bibliography entry

Russian routinely declines author surnames in possessive constructions, e.g. on a book spine or in a "Works by X" list:

> Романы **Толстого** — "the novels of Tolstoy"  *(illustrative)*

`Толстой → Толстого` is the standard genitive of a masculine surname in *-ой*. The case is selected because the surrounding construction is possessive. Other constructions select other cases: dative *Толстому*, accusative *Толстого* (homophonous with the genitive for animate nouns), instrumental *Толстым*. **(verified)** — Wade, *A Comprehensive Russian Grammar*.

A Russian bibliographic style ought to be able to ask for the genitive of an author name. It currently cannot.

### Worked example 3 — Basque inflected title in a possessive

This is the case that motivated CSL upstream issue #6369. In Basque, a title embedded in a possessive phrase takes a case suffix. For a title like "Etika" (Spinoza's *Ethics*):

> **Etikaren** sarrera — "the introduction to the *Ethics*"  🚧 (needs review)

The genitive marker *-(r)en* attaches to the title. **(verified)** — Hualde & Ortiz de Urbina, op. cit. The specific rendered string for any particular title needs to be checked with a Basque speaker before it ships in a locale file, because Basque combines case marking with the indefinite/definite distinction and with vowel-stem versus consonant-stem morphology, all of which can interact with the citation form.

### Worked example 4 — German genitive on a title

German is morphologically much lighter than Finnish or Basque, but even here the genitive shows up routinely:

> der zweite Band der ***Kritik der reinen Vernunft*** — "the second volume of the *Critique of Pure Reason*"  *(illustrative)*

In practice, German citation styles typically italicize the title and leave it in the nominative regardless of context, treating the title as a quoted block. **(verified)** — this is a stylistic convention, not a grammatical requirement.

The German example is included here for an important reason: **whether a language inflects titles in running prose is a stylistic choice**, not an automatic consequence of having a case system. A locale-level policy is needed.

### Worked example 5 — English-language style, foreign-language reference

Even an English-language style rendering a reference to a Russian-language work may want the original-language title in its citation-form case, not in whatever case it would naturally take in the running English sentence. In this scenario the *style's* language and the *title's* language disagree about whether to inflect, and the answer is generally: don't inflect — the title is being treated as a quoted citation block in English text.

This case is straightforward: if no inflection is requested, the renderer returns the base form. The interesting cases above are the ones where both the style language and the title language agree that inflection is required.

---

## What goes wrong without a design

Without a per-case data model, three failure modes occur in production:

1. **Wrong-case strings** — a Finnish style renders *Smith mukaan* instead of *Smithin mukaan*. Native readers perceive this as broken prose.
2. **Inconsistent workarounds** — different style authors hand-encode inflected forms in different fields (description, abstract, custom-string slots), making bibliographic data non-portable.
3. **Locale silos** — styles for inflecting languages get rewritten as one-off forks rather than expressing the language difference declaratively. The cost compounds with every new language.

A successful design avoids all three.

---

## Prior art

A scan of comparable systems turned up essentially nothing reusable. **(verified)** by inspection of the linked sources:

- **CSL 1.0 / CSL-JSON** ([specification](https://docs.citationstyles.org/en/stable/specification.html)): no concept of grammatical case on titles or names; treats them as opaque strings. The upstream issue [#6369](https://github.com/citation-style-language/styles/issues/6369) is the report that this gap exists.
- **citeproc-js / citeproc-rs / citation.js**: inherit CSL's opaque-string model.
- **biblatex / biber**: rich author/title metadata but no general per-case variants for titles or names. Two narrow precedents exist and are worth flagging:
  - **Locale-specific date macros for inflecting languages**, most visibly the Lithuanian localisation, where biblatex's `.lbx` files supply *derivation-based* inflected month names: the locale module receives a base month and emits the inflected form via locale-specific macros. This is the opposite of the design direction this spec considers (stored input variants, see Q1 Option A) and is therefore precedent in shape rather than approach.
  - **Gender-specific ordinal forms** (e.g. French *1<sup>er</sup>* / *1<sup>ère</sup>*) — closest architectural parallel to Citum's existing `MaybeGendered<T>`: a closed enum of variants resolved at render time. Useful as a precedent for the *shape* of the type, but the domain (ordinals) is small and closed, just like locale terms — the same scaling concern noted below for `MaybeGendered<T>` applies.
- **Zotero / better-bibtex**: no per-case input fields.
- **Pandoc-citeproc**: same as citeproc-js.
- **CLDR / ICU**: case-sensitive number and date formatters exist (ICU MessageFormat 2 supports a `:case=genitive` annotation), but neither CLDR nor ICU ships name/title declension data.

The closest in-repo precedent is **`MaybeGendered<T>`** in `crates/citum-schema-style/src/locale/types.rs`, which solves the analogous problem for *locale terms* (e.g. Spanish `editor` / `editora`). The shape generalizes, but the domains differ in two important ways:

- Locale terms are a **closed authored set** (a few dozen role labels, locator labels, and connectors). Titles and names are **open input data** — the universe of strings is unbounded.
- Gender on a locale term is a small fixed enum (masculine / feminine / neuter / common). Case on a title is a locale-declared list — possibly very long (15+ for Finnish) and structurally non-comparable across languages.

Title and name inflection is therefore a green-field design problem. Choices should be justified on first principles, not by appeal to convention.

---

## User-demand signal

The linguistic problem is real (see Worked Examples 1–3 above), but the user-facing demand for a solution in the current CSL tooling ecosystem is weaker than the linguistic case would suggest. A scan of the visible signal:

- **Zotero community forums** surface scattered threads from Finnish, Russian, and Hungarian users about wrong case marking in narrative citations and bibliography entries. The dominant resolutions are (a) manually editing the exported bibliography after the fact and (b) silently accepting ungrammatical output. Few threads escalate to a feature request, and none we are aware of has produced a sustained implementation effort upstream.
- **Juris-M**, the legal-citation Zotero fork that ships a measurably richer multilingual surface than upstream Zotero, handles **transliteration** of names and titles but deliberately does **not** handle grammatical-case inflection. That scope boundary is informative: the team most likely to invest in this gap has explicitly chosen not to.
- **CSL upstream issue [#6369](https://github.com/citation-style-language/styles/issues/6369)** is the most concrete public request, and it focuses on dates rather than names or titles. The bdarcus/csln prototype issue [#107](https://github.com/bdarcus/csln/issues/107) it traces back to has the same scope.

The interpretation: the problem is genuine, but the cost of grammatically-incorrect citation output (a few wrong word endings in an otherwise-readable bibliography) is low enough that affected users tolerate it, and the cost of producing per-case input data is currently high enough that no contributor has invested the sustained effort to fix it through the CSL stack. Until that asymmetry shifts — typically when a contributor with native-speaker fluency and locale-authoring motivation joins the project — speculative implementation is unlikely to produce ROI proportional to its maintenance cost.

This is the empirical basis for the staged approach in the Recommendation section above.

🚧 **Needs review:** a thorough survey of Polish, Czech, and Turkish user communities was not performed. If a stronger demand signal exists in one of those communities, it would meaningfully change the deferral calculus.

---

## Design questions

Six design questions structure the rest of this document. Each is presented with concrete options and tradeoffs. Under the staged approach recommended above, **Q1–Q3 (schema)** should be pressed to closure before the spec leaves Draft, and **Q4–Q5 (rendering)** are sketched but explicitly deferred to the future locale contributor who drives the renderer work. **Q6 (migration impact)** applies to both phases.

**No decisions are recommended in this draft** — the point is to make the choices visible. Concrete recommendations annotated below are starting points for the decision conversation, not commitments.

### Q1 — Input model: stored variants vs. derived

**Option A — Stored variants (explicit).** The input data carries per-case forms:

```yaml
title:
  base: "Etika"
  forms:
    genitive: "Etikaren"
    inessive: "Etikan"
```

**Option B — Engine derives forms (implicit).** The input carries only the base form; the engine applies morphological rules at render time, parameterized by locale and case.

| Criterion | Stored variants | Engine derives |
|---|---|---|
| Citum principle "explicit over magic" | ✔ matches | ✘ conflicts |
| Correctness for irregular forms | ✔ author-controlled | ✘ engine must encode every exception |
| Author burden | ✘ many fields to fill | ✔ one field |
| New language onboarding | ✔ no engine work | ✘ requires morphology rules per language |
| Failure mode | Missing form → fallback | Wrong rule → wrong output, silent |

**Recommendation in this draft:** pursue Option A in implementation; treat Option B as a possible future opt-in for languages with well-codified regular morphology. Open question: how authors actually get the inflected forms is a separate, downstream problem (could be Zotero plugins, manual entry, MT-assisted suggestion, …).

### Q2 — Schema surface

If Q1 lands on Option A, the schema needs a new type. Three candidate shapes:

```rust
// Sketch 1 — base + case map.
pub struct InflectedString {
    pub base: String,
    pub forms: HashMap<GrammaticalCase, String>,
}

// Sketch 2 — map-only, with an implicit "default" key.
pub struct InflectedString(HashMap<GrammaticalCase, String>);

// Sketch 3 — parallel to MaybeGendered<T>.
pub enum MaybeInflected<T> {
    Plain(T),
    Inflected { base: T, forms: HashMap<GrammaticalCase, T> },
}
```

Sketch 3 is the most ergonomic in Rust — `MaybeInflected<T>` composes with `MaybeGendered<T>` and the existing `Title` enum, and a plain string deserializes into `Plain(s)` without ceremony. The serde round-trip for the inflected variant needs design (probably an untagged enum, mirroring `MaybeGendered<T>`).

The `GrammaticalCase` type is the next question.

### Q3 — Locale binding for the case set

There is **no universal case vocabulary**. Different languages disagree on what cases *are*. Three approaches:

- **Hardcoded universal enum.** A fixed Rust enum covering "common" cases (nominative, genitive, dative, accusative, locative, instrumental, ablative, …). Compact and type-safe; fails for languages whose case categories don't fit (every language with a productive case system that isn't Latin or Russian).
- **Open string keys.** `HashMap<String, String>` with no enum. Maximally flexible; loses type-checking and risks typos (`"genitve"` vs `"genitive"`).
- **Locale-declared case set.** Each locale file declares the cases it recognizes. A `GrammaticalCase` value is then a `(locale_id, case_name)` pair, validated at locale-load time. Catches typos; supports any case inventory; the cost is a slightly more complex resolution path in the engine.

**Recommendation in this draft:** the locale-declared case set. It mirrors how `locator.kind` already works for compound locators (locale-declared vocabulary, validated on load).

### Q4 — How does a style request a specific case?

Several options, not mutually exclusive:

- **Explicit attribute on the template node.** `{ field: title, case: genitive }`. Verbose but unambiguous.
- **Implicit from surrounding macro.** A `possessive-of` macro requests its argument in the genitive automatically. Concise but couples grammar to template structure.
- **MF2 selector.** Inside a `pattern.*` message, dispatch on `$case` the way `pattern.page-range` dispatches on `$count`. Lets locale authors encode the case requirement next to the surface text.

The likely answer is a layered combination: a default case (probably nominative) at the field level, an explicit override on the template node, and MF2 selectors inside locale messages where the case requirement is determined by the surrounding text.

🚧 **Needs review:** is there a real Citum-native style that exercises all three layers, or do the layers reduce to two in practice? Worth asking a Finnish or Russian style author before locking the design in.

### Q5 — Fallback behavior

When a style requests a case that the input doesn't supply, the renderer's options are:

- **Silent fallback** — use the base / nominative form. Worst output is grammatically wrong text, identical to today's failure mode.
- **Render-time warning** — emit the base form and log a warning so the input maintainer can fix the data. Same visible output, better feedback loop.
- **Hard error** — refuse to render. Strict; appropriate where ungrammatical output is unacceptable (publication-grade Finnish text).

This should likely be **locale policy**: an English-language locale rendering a Russian-language title legitimately falls back silently to the citation form (the title is being quoted, not declined). A Finnish-language locale rendering a Finnish-language name should probably warn or error, because *Smith mukaan* is a real defect.

🚧 **Needs review:** is the right policy granularity per-locale, per-field-type (names vs titles), or per-case (some Finnish cases are stylistic, others are grammatically required)?

### Q6 — Migration impact

Most styles need no change. Most input data is in the nominative; inflection support is opt-in. Concrete migration scenarios:

- **Existing English styles** — no change. They never request a case; they always get the nominative.
- **Existing input data without per-case variants** — no change. `MaybeInflected<T>::Plain` round-trips identically to a bare string.
- **Existing CSL-derived styles** — no change. CSL has no case concept, so migration produces no case requests.
- **New Citum-native styles for inflecting languages** — opt in by referencing the new schema surface.

Confirmation needed: a sample Finnish or Russian style that round-trips a real bibliography under the proposed schema, with at least one narrative citation that exercises a non-nominative case.

---

## Open questions for follow-up research

These are flagged for explicit resolution before the spec leaves Draft status:

1. **Title inflection — stylistic or grammatical?** Worked example 4 (German) notes that even case-rich languages often treat titles as italicized nominative blocks. We need a per-language survey (or a representative one) to know which inflecting languages routinely decline titles in citations vs. quote them. The answer may differ from the analogous decision for names.
2. **Where do inflected forms come from?** Zotero, Crossref, OpenAlex etc. do not currently surface per-case variants. If the design lands on Option A (stored variants), the input data has to come from *somewhere*. Possible avenues: native-speaker authoring tools, optional MT-derived suggestions with human confirmation, locale-specific input plugins. None of these is in scope here, but the design should not assume them away.
3. **Capitalization interactions.** Some cases (Hungarian) attach affixes that may or may not preserve the original case of the noun, especially across hyphens. 🚧 (needs review)
4. **Interaction with transliteration.** A Russian author cited in an English text may be transliterated (*Tolstoj*, *Tolstoy*). If the Cyrillic source carries genitive *Толстого*, is the transliterated genitive *Tolstogo*? Always? Per-style? 🚧 (needs review)
5. **Interaction with disambiguation.** Citum's disambiguation logic compares author-year keys. If two cites carry the same author in different cases (*Smith*, *Smithin*) they must still resolve to the same disambiguation bucket. The base form is the obvious key, but this needs to be specified explicitly.
6. **Performance.** Adding a `HashMap` to every title and every author name multiplies the input size of a large bibliography by a non-trivial factor when used. Bench before committing to the layout. (Not a design blocker; an implementation concern.)

---

## Acceptance criteria for the design phase

This spec is considered ready to leave Draft under the staged plan (Recommendation section, above). The criteria split accordingly:

**Schema-side (pressed to closure before leaving Draft):**

- [ ] Q1 (input model), Q2 (schema surface), and Q3 (locale-binding for the case set) each have a decision recorded, with a one-paragraph rationale.
- [ ] At least one **native speaker** of an inflecting language has reviewed the worked examples and the proposed case-set mechanism, and corrected any 🚧 entries.
- [ ] The migration impact on at least one existing non-trivial style (e.g. a CSL-derived Finnish or Russian variant) has been concretely verified to be no-change for the schema-only landing.

**Rendering-side (sketched and explicitly deferred):**

- Q4 (style-language request mechanism) and Q5 (fallback behaviour) remain open. They should be revisited when a contributor commits to driving the renderer work; the spec should not be blocked on them.
- A sample Finnish or Russian style authored against the *full* renderer pipeline is not required for the schema-only landing.

**Cross-cutting:**

- [ ] Each open question 1–6 in the "Open questions for follow-up research" section has either an answer or a child bean that owns it.

---

## References

The following are standard reference grammars cited in this spec. Page-level claims would require checking against the books themselves; this spec cites them only at the level of "this language has case marking, and this is how the field describes it":

- Hualde, J. I., & Ortiz de Urbina, J. (eds.). (2003). *A Grammar of Basque*. Berlin: Mouton de Gruyter.
- Karlsson, F. (2017). *Finnish: A Comprehensive Grammar*. London: Routledge. (Earlier editions also valid.)
- Rounds, C. (2009). *Hungarian: An Essential Grammar*. London: Routledge.
- Wade, T. (2010). *A Comprehensive Russian Grammar* (3rd ed.). Oxford: Wiley-Blackwell.

Specific worked examples derived from grammar references are marked **(verified)** at the point of use. Examples shaped to convey the design problem rather than reproduce a citable sentence are marked **(illustrative)**.

For the Basque worked example specifically, the orthography of `-(r)en` and related case markers was also cross-checked against the Apertium project's open-source Basque grammar notes ([wiki.apertium.org/wiki/Basque_to_English](https://wiki.apertium.org/wiki/Basque_to_English)), which Citum uses as a secondary-source starting point for minority-language locales — see `docs/guides/AUTHORING_LOCALES.md` and the `locales/eu-ES.yaml` header.

---

## Changelog

- **0.2** — 2026-05-16 — Reframed scope around a staged approach: schema reservation now, renderer deferred until a motivated locale-specific contributor surfaces. Added the **Recommendation** section that formalises this position; added the **User-demand signal** section that supplies its empirical basis (Zotero forums, Juris-M's deliberate scope boundary, the dates-only focus of upstream CSL #6369). Expanded the **biblatex / biber** prior-art entry with the two specific precedents — Lithuanian derivation-based date macros and gender-specific ordinals. Split **Acceptance criteria** into schema-side (must close to leave Draft) and rendering-side (sketched, deferred). Version bumped from 0.1.
- **0.1** — 2026-05-16 — Initial draft (bean `csl26-1b4e`). Identifies the problem, surveys prior art, enumerates six design questions with options and tradeoffs, flags open questions for follow-up. No decisions recommended.
