---
# csl26-fsjy
title: 'Design: two-level style preset architecture'
status: completed
type: feature
priority: normal
created_at: 2026-03-17T10:38:21Z
updated_at: 2026-03-22T22:41:14Z
---

Design how presets work at both the options level (existing) and the style level (new), and how behavioral dependent styles like Turabian are represented as style-level preset variants rather than standalone YAML files.

## Problem

CSL has two classes of dependent styles:
1. Locale/title-only dependents — already handled by the alias system
2. Behavioral dependents (e.g. Turabian) — small formatting deviations from a
   parent (2-5 rules). Converting these as standalone YAML files is wasteful and
   creates maintenance burden.

Additionally, the top ~10 styles are compiled into the engine. There is no
formal concept of a 'style-level preset' — a named Style struct that can be
referenced without loading a YAML file.

## Two-level design (sketch)

**Level 1 — Options presets (existing):**
`use_preset: apa` on contributors/dates/titles. Fine-grained configuration
presets. Already implemented.

**Level 2 — Style presets (new):**
A named, compiled-in Style struct. Referenced as `preset: chicago-notes-18th`
at the top level of a style document (or in the wizard state). Behavioral
dependents become preset variants:
- `preset: chicago-notes-18th` → base Chicago Notes 18th
- `preset: chicago-notes-18th, variant: turabian` → Turabian overrides applied
  on top

The variant delta is small enough to be expressed as a patch on the base preset
rather than a full standalone style.

**Implications for the wizard:**
The Style Navigator's 'Closest match' banner names a preset (not a file).
'Use this' loads the preset into WizardState directly. A YAML file is only
produced when the user deviates from the preset — and even then, the output
can reference the preset by name and express only the delta.

**Implications for StyleInfo:**
`short_name` and `edition` fields (csl26-zy07) are the metadata surface for
style-level presets. The preset registry is the authoritative source; YAML
files that match a preset reference it by name.

## Questions to resolve

- Where does the preset registry live? (citum-schema-style, citum-engine, or
  citum-cli?) Likely citum-schema-style as compiled-in data.
- What is the variant delta format? Probably a partial Style struct (same serde
  shape, all fields optional, merged over the base).
- How many variants per preset are reasonable? Turabian is 1; are there others?
- Does the engine need to know about presets at render time, or only at
  authoring/wizard time?

## File Naming Convention

The filename slug is directly tied to the preset key decision. Once style-level
presets exist, the filename should derive from short_name + edition:
- Well-known styles: `{short-name-kebab}[-{edition-kebab}].yaml`
  e.g. `apa-7th.yaml`, `chicago-notes-18th-edition.yaml`, `mla-9th.yaml`
- Journal/publisher styles: keep `{publisher}-{variant}.yaml` (no short_name)

Getting the preset key shape right first avoids renaming twice. The rename wave
(csl26-nm8r or similar) should block on this design bean.
