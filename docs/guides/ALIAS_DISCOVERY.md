# Alias Discovery Guide

## What It Is

`scripts/find-alias-candidates.js` identifies independent CSL styles that render
identically (or near-identically) to a Citum registry builtin. The output is a
TSV report sorted by similarity score. Confirmed candidates become entries in
`registry/default.yaml` under the parent's `aliases:` array.

## How It Works

For each independent style in `styles-legacy/`:

1. Render the strict 12-scenario fixture (`tests/fixtures/citations-expanded.json` +
   `tests/fixtures/references-expanded.json`) through citeproc-js.
2. Score the output against pre-rendered fingerprints of every registry builtin,
   using `textSimilarity` from `oracle-utils.js` (normalized string comparison).
3. Report the best-matching target per candidate, plus per-scenario exact-match rates.

No Citum engine is involved — this is a CSL-to-CSL comparison.

## Running the Script

```bash
# Full corpus run (~3 minutes, ~2,827 candidates)
node scripts/find-alias-candidates.js

# Limit for smoke testing
node scripts/find-alias-candidates.js --limit 100

# Adjust similarity threshold and output path
node scripts/find-alias-candidates.js --threshold 0.98 --out /tmp/high-confidence.tsv

# Options
node scripts/find-alias-candidates.js --help
# --concurrency N   (default 8)
# --threshold F     (default 0.85, report only >= this)
# --limit N         (cap candidates, useful for testing)
# --out PATH        (default scripts/report-data/alias-candidates-YYYY-MM-DD.tsv)
```

## Review Protocol

| Score | Action |
|-------|--------|
| 1.000 | Strong candidate — check family membership, then alias |
| ≥ 0.98 | Candidate — manual spot-check of one citation + one bib entry |
| 0.85–0.98 | Low confidence — check style XML for structural clues |
| < 0.85 | Not reported |

**Before aliasing**, confirm the candidate is a true clone (not a same-family variant
with intentional differences the fixture doesn't exercise — see Known Blind Spots below).

### Web Confirmation (bean csl26-vepq)

A `--confirm-web` flag is tracked in bean **csl26-vepq**. It will query a search API
for each candidate above the threshold: `"<journal name>" citation style` and
`"<journal name>" author guidelines`. The results surface whether the journal's own
instructions name the parent style explicitly — reducing the need for per-row manual
lookup and providing a citable evidence URL in the TSV output.

Until that lands, for journal-named candidates: search `site:<publisher>.com
<journal-name> submission guidelines` and look for a named style.

## Known Fixture Blind Spots

The 12-scenario fixture does not exercise all behavioral dimensions. Candidates that
score 1.000 may still differ in areas the fixture doesn't cover:

| Fixture Gap | Affects |
|-------------|---------|
| No subsequent/repeated cites | `chicago-shortened-notes-bibliography-subsequent-{author,ibid,title}` |
| No note-context rendering | `modern-language-association-notes`, note-style Chicago variants |
| No bracket/delimiter shape variation | `american-medical-association-{brackets,parentheses}` |
| Limited archive-typed references | `chicago-shortened-notes-bibliography-archive-place-first` |

These 8 candidates were withheld from the 2026-04-19 alias wave pending fixture expansion.

### Expanding the Fixture

To close these gaps, add scenarios to `tests/fixtures/citations-expanded.json`:

- A reference cited twice in sequence (tests `subsequent-cite` behavior)
- A note-style citation (tests footnote vs in-text rendering)
- A citation with explicit bracket wrapping (`citation-number` types)
- An archive reference with both `archive` and `archive-place` populated

Re-run the script after expanding; withheld candidates that still score 1.000 can be
promoted to the alias list.

## Adding Confirmed Aliases

Edit `registry/default.yaml` and append to the parent's `aliases:` array:

```yaml
- id: elsevier-harvard
  aliases: [harvard, applied-clay-science, environmental-chemistry]
  builtin: elsevier-harvard
  ...
```

Commit the registry change with scope `registry`:

```
feat(registry): alias <N> <family> journal clones
```

Reference the dated TSV report and note the similarity score in the commit body.

## Historical Reports

Dated TSV snapshots are committed to `scripts/report-data/` and serve as an
audit trail for which candidates were reviewed at each wave.

| Date | File | Candidates | ≥0.85 hits |
|------|------|------------|------------|
| 2026-04-19 | `alias-candidates-2026-04-19.tsv` | 2,827 | 896 |
