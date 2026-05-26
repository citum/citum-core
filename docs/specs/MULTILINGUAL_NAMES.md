# Multilingual Names Specification

**Status:** Active
**Date:** 2026-05-26
**Related:** [`../architecture/MULTILINGUAL.md`](../architecture/MULTILINGUAL.md), [`../architecture/NAME_FORMATTING.md`](../architecture/NAME_FORMATTING.md), [`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md)

## Purpose

Define how Citum renders contributor names whose selected surface form carries
script-specific ordering and separator requirements. The first implementation
is motivated by Japanese Katakana names that need `・` in original order and
`、` in inverted order, but the contract is script-configured and not
Japanese-specific.

## Scope

In scope: selected multilingual name variants, script detection from rendered
name parts, script-specific native ordering, intra-name separators, inverted
sort separators, and precedence between template contributor options and
script options.

Out of scope: generating transliterations, locale-invented default separators,
CSL `name-part` affixes as a general formatting model, automatic behavior with
no style opt-in, and bibliography partitioning or collation rules.

## Design

Contributor name rendering is the composition of:

- the selected name variant, controlled by `options.multilingual.name-mode`
  and transliteration preferences;
- the detected script of the rendered given/family name parts;
- the effective name order from template contributor options, contributor
  config, and script-native ordering;
- the intra-name separator used for non-inverted names;
- the sort separator used for inverted names.

Styles opt in through `options.multilingual.scripts`:

```yaml
options:
  multilingual:
    name-mode: primary
    scripts:
      katakana:
        delimiter: "・"
        sort-separator: "、"
      cjk:
        use-native-ordering: true
        delimiter: ""
```

`delimiter` joins visible name parts in non-inverted order. For a Katakana
given/family pair, `delimiter: "・"` renders:

```text
マイケル・ジャクソン
```

`sort-separator` joins family and given parts when the effective contributor
rendering is inverted. For the same pair, `sort-separator: "、"` renders:

```text
ジャクソン、マイケル
```

`use-native-ordering: true` renders a matched script in family-first order when
the template has not explicitly requested another order. It is display order,
not CSL sort inversion. For native CJK names, `delimiter: ""` supports output
such as:

```text
北川善太郎
```

Script matching is deterministic. Exact script/category keys are preferred
over broader groups. The supported key set is:

| Key | Matches |
|---|---|
| `katakana` / `Kana` | All-Katakana name parts |
| `hiragana` / `Hira` | All-Hiragana name parts |
| `kana` / `Hrkt` | All-Hiragana, all-Katakana, or mixed Hiragana+Katakana |
| `han` / `Hani` | All-Han name parts |
| `hangul` / `Hang` | All-Hangul name parts |
| `cjk` | Any of the above, or mixes that don't resolve to a more specific key |

For a single-script name, the engine tries exact keys first (`katakana`,
`hiragana`, `han`, `hangul`), then ISO 15924 aliases, then the `kana` group
key, then `cjk`. For mixed-script names, the kana group (`Hiragana +
Katakana`, no Han or Hangul present) resolves to `kana`/`Hrkt`/`cjk`.
All other mixed CJK combinations resolve directly to `cjk`. Dominance-based
selection for Han+kana or other non-kana mixes is not implemented in this
version and is deferred to a future enhancement.

Template contributor options remain authoritative. A component-level
`sort-separator` overrides script-level `sort-separator`. Explicit
`name-order` or configured sort display still controls inversion before
script-native ordering is considered.

If no matching script config exists, Citum preserves existing behavior:
non-inverted names use spaces between name parts and inverted names use the
normal contributor `sort-separator` fallback.

## Implementation Notes

Script detection inspects the rendered `given`, `family`, particles, and
suffix after multilingual variant selection. Common or inherited punctuation
does not force a script match. The public contract is expressed in terms of
script categories, not codepoint ranges.

The rendering path should keep script-aware decisions local to contributor
assembly. Multilingual variant selection, bibliography partitioning, and
Unicode collation remain separate mechanisms.

The implementation uses the `unicode_script` crate for script classification,
which covers all Unicode planes including CJK Unified Ideographs Extension B+
(U+20000 and above). The public contract is defined in terms of script
categories, not codepoint ranges.

## Examples

### Latin name — no script config matches

```yaml
# style has no options.multilingual.scripts entry
```

```text
input : given="Jane", family="Smith"
output: Jane Smith          # given-first (default)
output: Smith, Jane         # inverted, uses normal ", " sort separator
```

Latin names fall through script detection with no match and preserve existing
behavior regardless of what script configs the style defines.

### Katakana name — with the example config above

```text
input : given="マイケル", family="ジャクソン"
output: マイケル・ジャクソン   # non-inverted, delimiter "・"
output: ジャクソン、マイケル   # inverted, sort-separator "、"
```

### Native CJK name — with use-native-ordering

```text
input : given="善太郎", family="北川"
output: 北川善太郎            # family-first, delimiter "" (no space)
```

## Precedence

`sort-separator` on a script config applies **only** when the engine renders
the name in inverted order. It is distinct from `delimiter`, which applies to
non-inverted rendering. Template contributor options always win:

- A component-level `sort-separator` attribute overrides the script-level
  `sort-separator`.
- An explicit template `name-order="given-family"` overrides
  `use-native-ordering: true` in the matched script config.

Mixed-script names (e.g., a family name in Han and a given name in Katakana)
fall through to `cjk` unless the mix is purely kana (Hiragana + Katakana),
which resolves to `kana`. Dominance-based selection for other mixes is not
implemented. Per-field or per-person overrides are out of scope for this
specification.

## Acceptance Criteria

- [ ] Existing styles preserve current name rendering unless they opt in with
      `options.multilingual.scripts`.
- [ ] Katakana non-inverted names can render with `・` between given and family
      parts.
- [ ] Katakana inverted names can render with `、` between family and given
      parts.
- [ ] Native CJK names can render in native family-first order without an
      inserted space.
- [ ] Template-level `sort-separator` overrides script-level `sort-separator`.
- [ ] Explicit template `name-order` overrides script `use-native-ordering`.
- [ ] Generated schemas expose script-level `sort-separator`.

## Changelog

- 2026-05-26: Initial draft.
- 2026-05-26: Marked active for the initial script-specific separator implementation.
