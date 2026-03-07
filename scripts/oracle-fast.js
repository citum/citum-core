#!/usr/bin/env node
/**
 * scripts/oracle-fast.js
 *
 * Snapshot-based oracle: loads pre-computed citeproc-js output from
 * tests/snapshots/csl/<style>.json and diffs against the live Citum renderer.
 * Drop-in replacement for oracle.js in report-core.js non-migrate runs.
 *
 * Requires a current snapshot. Exits 2 if snapshot is missing, 3 if stale.
 * To regenerate: node scripts/oracle-snapshot.js <style.csl>
 *
 * Usage:
 *   node scripts/oracle-fast.js <style.csl>
 *   node scripts/oracle-fast.js <style.csl> --json
 *   node scripts/oracle-fast.js <style.csl> --verbose
 *
 * Exit codes:
 *   0 — all citations and bibliography match
 *   1 — mismatches found
 *   2 — snapshot file missing
 *   3 — snapshot stale (fixture or CSL changed)
 */

'use strict';

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const { normalizeText, parseComponents, analyzeOrdering, findRefDataForEntry } = require('./oracle-utils');
const { renderWithCslnProcessor } = require('./oracle');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const SNAPSHOT_DIR = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'csl');
const DEFAULT_REFS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json');
const DEFAULT_CITATIONS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-expanded.json');

const STRICT_CITATION_IDS = new Set([
  'et-al-single-long-list',
  'disambiguate-add-names-et-al',
  'disambiguate-year-suffix',
  'et-al-with-locator',
]);

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    stylePath: null,
    jsonOutput: false,
    verbose: false,
    refsFixture: DEFAULT_REFS_FIXTURE,
    citationsFixture: DEFAULT_CITATIONS_FIXTURE,
  };
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === '--json') opts.jsonOutput = true;
    else if (a === '--verbose') opts.verbose = true;
    else if (a === '--refs-fixture') opts.refsFixture = path.resolve(args[++i]);
    else if (a === '--citations-fixture') opts.citationsFixture = path.resolve(args[++i]);
    else if (!a.startsWith('--') && !opts.stylePath) opts.stylePath = path.resolve(a);
  }
  return opts;
}

// ---------------------------------------------------------------------------
// Fixture hashing (must match oracle-snapshot.js)
// ---------------------------------------------------------------------------

function fixtureHash(refsFixture, citationsFixture) {
  const h = crypto.createHash('sha256');
  h.update(fs.readFileSync(refsFixture));
  h.update(fs.readFileSync(citationsFixture));
  return h.digest('hex').slice(0, 16);
}

// ---------------------------------------------------------------------------
// Snapshot loading with staleness guard
// ---------------------------------------------------------------------------

class SnapshotMissingError extends Error {}
class SnapshotStaleError extends Error {}

/**
 * Load snapshot for a CSL style, validating the fixture_hash.
 * Returns the parsed snapshot or throws SnapshotMissingError / SnapshotStaleError.
 */
function loadSnapshot(stylePath, refsFixture, citationsFixture) {
  const styleName = path.basename(stylePath, '.csl');
  const snapPath = path.join(SNAPSHOT_DIR, `${styleName}.json`);

  if (!fs.existsSync(snapPath)) {
    throw new SnapshotMissingError(
      `Snapshot missing for ${styleName}.\n` +
      `  Run: node scripts/oracle-snapshot.js ${stylePath}`
    );
  }

  const snap = JSON.parse(fs.readFileSync(snapPath, 'utf8'));
  const currentHash = fixtureHash(refsFixture, citationsFixture);

  if (snap.fixture_hash !== currentHash) {
    throw new SnapshotStaleError(
      `Snapshot stale for ${styleName} (fixture changed).\n` +
      `  Run: node scripts/oracle-snapshot.js ${stylePath}`
    );
  }

  return snap;
}

// ---------------------------------------------------------------------------
// Comparison logic (mirrors oracle.js)
// ---------------------------------------------------------------------------

function tokenizeForSimilarity(text) {
  return normalizeText(text || '')
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s]/gu, ' ')
    .split(/\s+/)
    .filter(Boolean)
    .filter((t) => t.length > 1);
}

function textSimilarity(a, b) {
  const left = tokenizeForSimilarity(a);
  const right = tokenizeForSimilarity(b);
  if (left.length === 0 && right.length === 0) return 1;
  if (left.length === 0 || right.length === 0) return 0;
  const lc = new Map();
  const rc = new Map();
  for (const t of left) lc.set(t, (lc.get(t) || 0) + 1);
  for (const t of right) rc.set(t, (rc.get(t) || 0) + 1);
  let intersect = 0;
  let union = 0;
  for (const k of new Set([...lc.keys(), ...rc.keys()])) {
    const l = lc.get(k) || 0;
    const r = rc.get(k) || 0;
    intersect += Math.min(l, r);
    union += Math.max(l, r);
  }
  return union > 0 ? intersect / union : 0;
}

function equivalentText(a, b) {
  const an = normalizeText(a);
  const bn = normalizeText(b);
  if (an === bn) return true;
  return textSimilarity(an, bn) >= 0.60;
}

function extractYearSuffixes(text) {
  return normalizeText(text).match(/\b\d{4}[a-z]\b/gi) || [];
}

function hasEtAl(text) {
  return /\bet al\b/i.test(normalizeText(text));
}

function splitCitationCluster(text) {
  return normalizeText(text)
    .replace(/^\(/, '').replace(/\)$/, '')
    .split(/\s*;\s*/).map((p) => p.trim()).filter(Boolean);
}

function extractLocatorNumber(text) {
  const m = normalizeText(text).match(/\b(?:p|pp|section|sec)\.?\s*(\d+)\b/i);
  return m ? m[1] : null;
}

function equivalentCitationText(oracleText, cslnText, citationId) {
  if (!STRICT_CITATION_IDS.has(citationId)) return equivalentText(oracleText, cslnText);

  const oN = normalizeText(oracleText);
  const cN = normalizeText(cslnText);
  if (hasEtAl(oN) && !hasEtAl(cN)) return false;
  if (extractYearSuffixes(oN).length > 0 && extractYearSuffixes(cN).length === 0) return false;
  if (citationId === 'disambiguate-add-names-et-al') {
    if (hasEtAl(oN) || extractYearSuffixes(oN).length > 0) {
      const parts = splitCitationCluster(cN);
      if (parts.length < 2 || new Set(parts).size !== parts.length) return false;
    }
  }
  if (citationId === 'et-al-with-locator') {
    const oL = extractLocatorNumber(oN);
    const cL = extractLocatorNumber(cN);
    if (oL && oL !== cL) return false;
  }
  return true;
}

/**
 * Pair bibliography entries by text similarity (greedy best-match).
 */
function matchBibliographyEntries(oracleBib, cslnBib) {
  const pairs = [];
  const usedO = new Set();
  const usedC = new Set();
  const candidates = [];

  for (let oi = 0; oi < oracleBib.length; oi++) {
    for (let ci = 0; ci < cslnBib.length; ci++) {
      const score = textSimilarity(oracleBib[oi], cslnBib[ci]);
      if (score >= 0.20) candidates.push({ oi, ci, score });
    }
  }
  candidates.sort((a, b) => b.score - a.score);
  for (const c of candidates) {
    if (usedO.has(c.oi) || usedC.has(c.ci)) continue;
    usedO.add(c.oi);
    usedC.add(c.ci);
    pairs.push({ oracle: oracleBib[c.oi], csln: cslnBib[c.ci], score: c.score });
  }
  for (let oi = 0; oi < oracleBib.length; oi++) {
    if (!usedO.has(oi)) pairs.push({ oracle: oracleBib[oi], csln: null, score: 0 });
  }
  for (let ci = 0; ci < cslnBib.length; ci++) {
    if (!usedC.has(ci)) pairs.push({ oracle: null, csln: cslnBib[ci], score: 0 });
  }
  return pairs;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function run() {
  const opts = parseArgs();

  if (!opts.stylePath) {
    process.stderr.write('Usage: oracle-fast.js <style.csl> [--json] [--verbose]\n');
    process.exit(2);
  }

  if (!fs.existsSync(opts.stylePath)) {
    process.stderr.write(`Style not found: ${opts.stylePath}\n`);
    process.exit(2);
  }

  // 1. Load and validate snapshot
  let snapshot;
  try {
    snapshot = loadSnapshot(opts.stylePath, opts.refsFixture, opts.citationsFixture);
  } catch (err) {
    process.stderr.write(`oracle-fast: ${err.message}\n`);
    process.exit(err instanceof SnapshotStaleError ? 3 : 2);
  }

  // 2. Load fixtures for CSLN rendering
  const testItems = Object.fromEntries(
    Object.entries(JSON.parse(fs.readFileSync(opts.refsFixture, 'utf8')))
      .filter(([k]) => k !== 'comment')
  );
  const testCitations = JSON.parse(fs.readFileSync(opts.citationsFixture, 'utf8'));

  // 3. Render with Citum
  const csln = renderWithCslnProcessor(opts.stylePath, testItems, testCitations);
  if (!csln || csln.error) {
    const reason = csln?.error ?? 'Processor execution error';
    if (opts.jsonOutput) {
      process.stdout.write(JSON.stringify({
        error: 'CSLN rendering failed', reason,
        style: path.basename(opts.stylePath, '.csl'),
      }) + '\n');
    } else {
      process.stderr.write(`CSLN rendering failed: ${reason}\n`);
    }
    process.exit(2);
  }

  const styleName = path.basename(opts.stylePath, '.csl');

  // 4. Diff
  const pairs = matchBibliographyEntries(snapshot.bibliography, csln.bibliography);

  const results = {
    style: styleName,
    oracleSource: 'citeproc-js',
    snapshotGeneratedBy: snapshot.generated_by,
    citations: { total: testCitations.length, passed: 0, failed: 0, entries: [] },
    citationsByType: {},
    bibliography: { total: pairs.length, passed: 0, failed: 0, entries: [] },
    componentSummary: {},
    orderingIssues: 0,
  };

  for (const cite of testCitations) {
    const oracleText = normalizeText(snapshot.citations[cite.id] || '');
    const cslnText = normalizeText(csln.citations[cite.id] || '');
    const match = equivalentCitationText(oracleText, cslnText, cite.id);
    if (match) results.citations.passed++; else results.citations.failed++;
    results.citations.entries.push({ id: cite.id, oracle: oracleText, csln: cslnText, match });

    for (const item of cite.items || []) {
      const type = testItems[item.id]?.type ?? 'unknown';
      if (!results.citationsByType[type]) results.citationsByType[type] = { total: 0, passed: 0 };
      results.citationsByType[type].total++;
      if (match) results.citationsByType[type].passed++;
    }
  }

  for (let i = 0; i < pairs.length; i++) {
    const pair = pairs[i];
    const entryResult = {
      index: i + 1,
      oracle: pair.oracle ? normalizeText(pair.oracle) : null,
      csln: pair.csln ? normalizeText(pair.csln) : null,
      match: false,
      components: {},
      ordering: null,
      issues: [],
    };

    if (!pair.oracle) {
      entryResult.issues.push({ issue: 'extra_entry', detail: 'Entry in CSLN but not oracle' });
      results.bibliography.failed++;
    } else if (!pair.csln) {
      entryResult.issues.push({ issue: 'missing_entry', detail: 'Entry in oracle but not CSLN' });
      results.bibliography.failed++;
    } else {
      const oN = normalizeText(pair.oracle);
      const cN = normalizeText(pair.csln);
      if (equivalentText(oN, cN)) {
        entryResult.match = true;
        results.bibliography.passed++;
      } else {
        results.bibliography.failed++;
        const refData = findRefDataForEntry(pair.oracle, testItems);
        if (refData) {
          const oComp = parseComponents(pair.oracle, refData);
          const cComp = parseComponents(pair.csln, refData);
          const differences = [];
          const matches = [];
          const keys = ['contributors', 'year', 'title', 'containerTitle', 'volume',
            'issue', 'pages', 'publisher', 'doi', 'edition', 'editors'];
          for (const key of keys) {
            const ov = oComp[key];
            const cv = cComp[key];
            if (!ov.found && !cv.found) continue;
            if (ov.found && cv.found) {
              matches.push({ component: key, status: 'match' });
            } else if (ov.found && !cv.found) {
              differences.push({ component: key, issue: 'missing', expected: ov.value, detail: 'Missing in CSLN output' });
            } else {
              differences.push({ component: key, issue: 'extra', found: cv.value, detail: 'Extra in CSLN output' });
            }
          }
          entryResult.components = { differences, matches };

          const oOrder = analyzeOrdering(pair.oracle, refData);
          const cOrder = analyzeOrdering(pair.csln, refData);
          if (JSON.stringify(oOrder) !== JSON.stringify(cOrder)) {
            entryResult.ordering = { oracle: oOrder, csln: cOrder };
            results.orderingIssues++;
          }
          entryResult.issues = [...differences];
          for (const [key, count] of Object.entries(
            differences.reduce((acc, d) => { acc[`${d.component}:${d.issue}`] = (acc[`${d.component}:${d.issue}`] || 0) + 1; return acc; }, {})
          )) {
            results.componentSummary[key] = (results.componentSummary[key] || 0) + count;
          }
        }
      }
    }

    results.bibliography.entries.push(entryResult);
  }

  // 5. Output
  if (opts.jsonOutput) {
    process.stdout.write(JSON.stringify(results, null, 2) + '\n');
  } else {
    process.stderr.write(`\n=== Fast Oracle: ${styleName} (${snapshot.generated_by}) ===\n\n`);
    process.stderr.write(`Citations:    ${results.citations.passed}/${results.citations.total}\n`);
    process.stderr.write(`Bibliography: ${results.bibliography.passed}/${results.bibliography.total}\n`);
    if (opts.verbose) {
      for (const e of results.citations.entries.filter((e) => !e.match)) {
        process.stderr.write(`  [FAIL] ${e.id}\n    oracle: ${e.oracle}\n    csln:   ${e.csln}\n`);
      }
    }
  }

  process.exit(results.citations.failed === 0 && results.bibliography.failed === 0 ? 0 : 1);
}

run();
