# GB/T 7714—2025 CSL-M fixture provenance

These fixtures are adapted from
[`zotero-chinese/styles`](https://github.com/zotero-chinese/styles) at commit
[`363713c6cea19568a863944607a4ea7391369b73`](https://github.com/zotero-chinese/styles/commit/363713c6cea19568a863944607a4ea7391369b73).

The upstream CSL styles are licensed under
[Creative Commons Attribution-ShareAlike 3.0](https://creativecommons.org/licenses/by-sa/3.0/)
and name Zeping Lee as their author. The repository root is distributed under
the same license. These copies retain the upstream metadata and rights notice.

## Source mappings

- `gb-t-7714-2025-numeric.csl` is an exact, renamed copy of
  `src/GB-T-7714—2025（顺序编码，双语）/GB-T-7714—2025（顺序编码，双语）.csl`.
- `gb-t-7714-2025-author-date.csl` is an exact, renamed copy of
  `src/GB-T-7714—2025（著者-出版年，双语）/GB-T-7714—2025（著者-出版年，双语）.csl`.
- `gb-t-7714-2025-note.csl` is an exact, renamed copy of
  `src/GB-T-7714—2025（注释，双语）/GB-T-7714—2025（注释，双语）.csl`.
- `../test-items-library/gb-t-7714-2025.json` is an exact copy of the shared
  upstream `items.json`. All three source directories contain identical item
  data. Its SHA-256 is
  `83110354a16927351b50d9d3b7988ee119f859017157226dd07a9cc8eb531b7d`.

## Citation fixture adaptations

The three `*-citations.json` files preserve the item IDs, item order, cluster
order, and locators from the corresponding upstream `cites.json`, while adding
stable scenario IDs required by Citum's oracle harness.

- The numeric source's single eight-item cluster becomes one `items` scenario.
- The author-date source's eight clusters become eight ordered `items`
  scenarios.
- The note source's eight clusters become one ordered `clusters` scenario so
  repeated-note position tracking is preserved. Its final string locator and
  sibling label are represented by Citum's equivalent structured locator
  object.

No output strings from upstream `metadata.json` are copied. Oracle expectations
are generated from the pinned CSL-M sources and compared with the adapted
Citum fixture inputs.
