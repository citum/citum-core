# Role-Label Defaults

Status: Active

## Overview

Resolves div-012: the engine previously hardcoded an APA-shaped short-form
suffix (`" (ed.)"`) for seven contributor roles (editor, chair, translator,
interviewer, director, illustrator, composer) whenever a `Long`-form
contributor component had no explicit `label:` or configured role preset —
in both citation and bibliography context, for every style.

Style-guide research (PR #1017, divergence register div-012) established:

1. Role labels are a **bibliography-only** convention in every examined
   style (APA, MLA, Chicago, Vancouver/NLM). No style carries a role label
   into the in-text citation.
2. The role set and label form are **per-style**: APA labels editor only,
   abbreviated; MLA labels a broader set in word form; Chicago prefers
   post-title phrasing and labels none.

## Design

The engine-wide implicit default is **removed**. A style that wants
automatic role labels declares a named bundle:

```yaml
contributors:
  role:
    defaults: apa   # or: mla, none
```

`RoleLabelDefaults` (crates/citum-schema-style/src/options/contributors.rs):

| Bundle | Roles labeled | Preset | Rendered shape |
|---|---|---|---|
| `none` (= unset) | — | — | no automatic label |
| `apa` | editor | `short-suffix` | `" (ed.)"` (en-US short term; capitalization needs an explicit `text-case`) |
| `mla` | editor, translator, director, illustrator, interviewer | `long-suffix` | `", editor"` |

Resolution order in `resolve_role_labels`
(crates/citum-engine/src/values/contributor/labels.rs), unchanged except
for the last step:

1. Explicit component `label:` config — context-independent.
2. `role.omit` suppression (decorative labels only; verb forms keep their
   structural phrase).
3. Configured role presets (`role.preset`, `role.roles.<role>.preset`,
   `role.form`) — context-independent: explicit configuration is the
   style's own choice.
4. **`role.defaults` bundle — bibliography context only** (`RenderOptions::
   context == RenderContext::Bibliography`), and never for verb/verb-short
   forms.

### Future extensibility

The bundles currently map each role to a single suffix-shaped preset. MLA
also uses labels as preceding descriptors in some Contributor-element
positions; that case is already expressible today via per-role presets
(`role.preset`, `role.roles.<role>.preset`, `role.form`) and verb/verb-short
forms, which the bundle never overrides. If the `mla` bundle is later
extended beyond simple long-suffixes, it should evolve into a per-role
"default label strategy" (placement + form) rather than a suffix shape —
the schema surface (`defaults:` as an enum) leaves room for that without
breaking existing styles.

## Compatibility

A raw-output diff (not just pass counts — the report normalizes both
sides, e.g. `" (Eds.)"` ↔ `"editors"`, so its `match` field tolerates
role-label differences) classified every affected fixture entry against
its citeproc-js oracle:

- Interviewer/director suffixes (Chicago, Taylor & Francis, MHRA,
  Hart's): the oracle has no label — removal moved output toward the
  oracle. Kept.
- Editor labels (elsevier-with-titles family, hawaii/HICSS,
  springer-basic): the CSL sources request explicit labels
  (`" (Eds.)"`, `" (eds)"`, `", eds."`). These styles had silently
  relied on the engine default (rendering a not-quite-right `" (eds.)"`)
  — they now declare component-level `label:` config matching their CSL
  exactly, including affixes via the `RoleLabel` `prefix`/`suffix`
  fields added for this (mirroring CSL 1.0 `cs:label` affixes).

Styles that want automatic labels opt in via a `defaults` bundle,
per-role presets, or explicit component labels.

`citum-migrate` does not rely on the implicit default: CSL 1.0 expresses
labels explicitly (`cs:label`), which migrates to explicit `label:` config.

## References

- div-012, docs/adjudication/DIVERGENCE_REGISTER.md
- Bean csl26-xve4; audit finding 16b (2026-07-04 engine review part 2)
- div-011 (`SubstituteTitleQuoteMode`) — precedent for style-gated defaults
