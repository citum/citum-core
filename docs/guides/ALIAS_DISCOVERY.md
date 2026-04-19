# Alias Discovery Guide

## What It Is

`scripts/find-alias-candidates.js` identifies independent CSL styles that render
identically (or near-identically) to a Citum registry builtin. The output is a
TSV report sorted by similarity score. Confirmed candidates become entries in
`registry/default.yaml` under the parent's `aliases:` array.

## How It Works

For each independent style in `styles-legacy/`:

1. Render the fixture scenarios (`tests/fixtures/citations-expanded.json` +
   `tests/fixtures/references-expanded.json`) through citeproc-js using `processCitationCluster`
   for position-aware rendering (enables ibid/subsequent detection).
2. Score the output against pre-rendered fingerprints of every registry builtin:
   - `citation_match` / `bib_match`: exact string equality (post-normalizeText) — detects
     structural differences like bracket vs parenthesis notation.
   - `similarity`: bag-of-words textSimilarity for finer-grained token-level matching.
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

## Known Fixture Blind Spots & Fixes (2026-04-19)

As of 2026-04-19, the following blind spots have been resolved:

| Previously Blind Spot | Fix | Result |
|----------------------|-----|--------|
| No subsequent/repeated cites | Added `subsequent-same-item` scenario with `processCitationCluster` position tracking | Chicago subsequent-ibid now scores 0.9744, dropping citation_match below 1.0 |
| Bracket/delimiter shape variation | Fixed `citation_match` to use exact string equality (not bag-of-words) | AMA-brackets and AMA-parentheses now score citation_match 0.0000 ≠ 1.0 |
| Note-context rendering | Added `archive-single` scenario with archive-typed ITEM-34 | Structural differences in archive-place handling now visible |
| `modern-language-association-notes` issue | Promoted to confirmed alias in registry (identical citation XML to MLA base) | Excluded from candidate list; no longer a false positive |

### Previously Unresolved Blind Spot

| Fixture Gap | Notes |
|-------------|-------|
| Note-style vs in-text rendering | `modern-language-association-notes` now in registry; remaining note-style variants would require note-context fixture scenario |

These gaps have been closed by fixture expansion and scoring tightening. Re-run the script
with these changes; previous withheld candidates are now properly scored.

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

| Date | File | Candidates | ≥0.85 hits | Notes |
|------|------|------------|------------|-------|
| 2026-04-19 | `alias-candidates-2026-04-19.tsv` | 2,823 | 2,307 | Fixed: processCitationCluster position tracking + exact-string citation_match scoring |
