---
# csl26-zzrl
title: Fix two remaining hardcoded role-label anti-patterns (RSC + AFS)
status: completed
type: bug
priority: normal
created_at: 2026-06-18T15:05:58Z
updated_at: 2026-06-18T16:38:30Z
---

Two hardcoded English role-label strings could not be fixed during csl26-6bul
work because the locale term forms don't exactly reproduce the original punctuation.

## Pattern 1 — Royal Society of Chemistry (RSC)

**File:** `styles/royal-society-of-chemistry.yaml`

**Line ~98** (inside `chapter.add` diff):
```yaml
        - component:
            contributor: editor
            form: short
            name-order: given-first
            prefix: ', editors. '
          before:
            variable: publisher-place
```

**Output:** `, editors. Smith A, Jones B` — label BEFORE names (non-standard), with trailing `. ` as structural separator.

**Problem:** No locale form gives `"editors. "` exactly.
- `long` = `"editors"` (no dot)
- `short` = `"eds."` (abbreviated, has dot)
- `verb-short` = `"ed. by"` (wrong meaning)
The correct fix needs a two-component group: `term: editor form: long suffix: '. '` followed by the contributor, wrapped with `prefix: ', '`. This changes the diff shape and needs testing.

## Pattern 2 — American Fisheries Society (AFS)

**File:** `styles/american-fisheries-society.yaml`

**Line ~163–164** (bibliography template, base template section):
```yaml
  - contributor: editor
    form: verb
    name-order: given-first
    prefix: " in "
  - variable: publisher
    prefix: ", editor. "
```

**Problem:** `prefix: ", editor. "` is on a `variable: publisher` component, not a contributor. A role label on a variable has no locale-term equivalent. Need to read the expected output from the original CSL (http://www.zotero.org/styles/american-fisheries-society) to understand the intended format before proposing a fix. It may be a pre-existing bug in the migrated style.

## Fix approach

For RSC: Replace the single contributor component (in the `add` diff) with a group:
```yaml
        - component:
            group:
            - term: editor
              form: long
              suffix: '. '
            - contributor: editor
              form: short
              name-order: given-first
            prefix: ', '
          before:
            variable: publisher-place
```

For AFS: Requires oracle comparison against the original CSL first.

## Resolution

Completed in `fix(styles): align RSC and AFS role labels`.

The AFS "Pattern 2" diagnosis above was wrong. The primary authority is the AFS
reference guide, not the migrated CSL shape, and the guide consistently places
the role label as a suffix on the contributor group: edited books render as
`Name, editor(s). Year...`, and chapters render as
`Pages ... in Name, editor(s). Container title...`.

The fix updates AFS to use the guide document directly and rewrites the
book/chapter contributor-role structure so role labels attach to contributors
instead of publisher metadata. RSC now uses the schema-native contributor
role-label API rather than hardcoded English role-label text.
