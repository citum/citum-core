#!/usr/bin/env node
/**
 * oracle-native.js — Snapshot-based oracle for native Citum-created styles.
 *
 * These styles have no CSL equivalent to diff against. Instead we:
 *   1. Run `citum render refs` against a fixed fixture
 *   2. Compare all output sections (citations + bibliography) to a stored snapshot
 *   3. In bootstrap mode (snapshot absent) write it and report fidelity 1.0
 *
 * The citationsByType map is derived from the fixture itself: for each
 * bibliography group, the types of the member references are credited with
 * a pass or fail based on whether that group's line matches the snapshot.
 *
 * Usage:
 *   node oracle-native.js <style-name> <style-yaml> <fixture-path> <snapshot-dir>
 *
 * Exit codes:
 *   0 — perfect match (or bootstrap)
 *   1 — mismatch or render error (JSON still written to stdout)
 *   2 — bad arguments
 */

'use strict';

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');
const { spawnSync } = require('child_process');

const [styleName, styleYamlPath, fixturePath, snapshotDir] = process.argv.slice(2);

if (!styleName || !styleYamlPath || !fixturePath || !snapshotDir) {
  process.stderr.write(
    'Usage: oracle-native.js <style-name> <style-yaml> <fixture-path> <snapshot-dir>\n'
  );
  process.exit(2);
}

const projectRoot = path.resolve(__dirname, '..');

/** Run `citum render refs` and return { stdout, stderr, status } */
function renderRefs() {
  const result = spawnSync(
    'cargo',
    ['run', '--quiet', '--bin', 'citum', '--', 'render', 'refs', '-b', fixturePath, '-s', styleYamlPath],
    { cwd: projectRoot, encoding: 'utf8', timeout: 120000 }
  );
  return result;
}

/**
 * Extract non-empty lines from a labelled section in the rendered output.
 * Returns lines between the label (e.g. "BIBLIOGRAPHY:") and the next
 * blank line group / next section header.
 */
function parseSection(text, header) {
  const lines = text.split('\n');
  const start = lines.findIndex((l) => l.trim() === header);
  if (start === -1) return [];
  return lines
    .slice(start + 1)
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !l.endsWith(':'));
}

/**
 * Read the fixture YAML and return a mapping of set-position → set of
 * reference types included in that group.
 *
 * citum class+type → CSL-like type string used in citationsByType.
 */
const CLASS_TYPE_MAP = {
  'monograph:book': 'book',
  'monograph:report': 'report',
  'monograph:thesis': 'thesis',
  'monograph:webpage': 'webpage',
  'monograph:document': 'document',
  'serial-component:article': 'article-journal',
  'serial-component:post': 'post',
  'serial-component:review': 'review',
  'collection-component:chapter': 'chapter',
  'collection-component:document': 'paper-conference',
};

function buildFixtureTypeIndex(fixturePath) {
  let fixture;
  try {
    fixture = yaml.load(fs.readFileSync(fixturePath, 'utf8'));
  } catch {
    return { refTypes: {}, setGroups: [] };
  }

  // Map refId → CSL-like type string
  const refTypes = {};
  for (const ref of (fixture.references || [])) {
    const cls = ref.class || '';
    const type = ref.type || '';
    const key = `${cls}:${type}`;
    refTypes[ref.id] = CLASS_TYPE_MAP[key] || type;
  }

  // Collect ordered set groups (sets come first, then standalone refs)
  const setsMap = fixture.sets || {};
  const allSetMembers = new Set(Object.values(setsMap).flat());

  const setGroups = [];

  // Sets first (in YAML key order)
  for (const [, members] of Object.entries(setsMap)) {
    setGroups.push(members.map((id) => refTypes[id] || 'unknown'));
  }

  // Standalones next
  for (const ref of (fixture.references || [])) {
    if (!allSetMembers.has(ref.id)) {
      setGroups.push([refTypes[ref.id] || 'unknown']);
    }
  }

  return { refTypes, setGroups };
}

/**
 * Build citationsByType from a list of bibliography entries and the
 * parallel set-type groups from the fixture.
 */
function buildCitationsByType(bibEntries, setGroups) {
  const byType = {};
  const n = Math.min(bibEntries.length, setGroups.length);

  for (let i = 0; i < n; i++) {
    const entry = bibEntries[i];
    const types = [...new Set(setGroups[i])]; // unique types in this group
    for (const t of types) {
      if (!byType[t]) byType[t] = { total: 0, passed: 0 };
      byType[t].total += 1;
      if (entry.match) byType[t].passed += 1;
    }
  }

  return byType;
}

/** Emit JSON result to stdout and exit */
function emit(result, exitCode) {
  process.stdout.write(JSON.stringify(result));
  process.exit(exitCode);
}

const rendered = renderRefs();

if (rendered.status !== 0 || rendered.error) {
  const errMsg = rendered.stderr
    ? rendered.stderr.trim().split('\n').slice(-3).join(' ')
    : (rendered.error?.message ?? 'unknown error');
  emit(
    {
      error: `citum render failed (exit ${rendered.status}): ${errMsg}`,
      citations: { passed: 0, total: 0 },
      bibliography: { passed: 0, total: 0, entries: [] },
      citationsByType: {},
      componentSummary: {},
    },
    1
  );
}

const actualText = (rendered.stdout || '').trim();
const actualBibEntries = parseSection(actualText, 'BIBLIOGRAPHY:');
const actualNonIntegral = parseSection(actualText, 'CITATIONS (Non-Integral):');
const actualIntegral = parseSection(actualText, 'CITATIONS (Integral):');
const snapshotPath = path.join(snapshotDir, `${styleName}.txt`);

const { setGroups } = buildFixtureTypeIndex(fixturePath);

// Bootstrap mode: snapshot absent — write it and report perfect fidelity
if (!fs.existsSync(snapshotPath)) {
  fs.mkdirSync(snapshotDir, { recursive: true });
  fs.writeFileSync(snapshotPath, actualText, 'utf8');
  const bibN = actualBibEntries.length;
  const citN = actualNonIntegral.length + actualIntegral.length;
  const bibEntries = actualBibEntries.map((e) => ({ expected: e, actual: e, match: true }));
  const citationsByType = buildCitationsByType(bibEntries, setGroups);
  emit(
    {
      citations: { passed: citN, total: citN },
      bibliography: { passed: bibN, total: bibN, entries: bibEntries },
      citationsByType,
      componentSummary: {},
    },
    0
  );
}

// Comparison mode: diff actual vs snapshot
const expectedText = fs.readFileSync(snapshotPath, 'utf8').trim();
const expectedBibEntries = parseSection(expectedText, 'BIBLIOGRAPHY:');
const expectedNonIntegral = parseSection(expectedText, 'CITATIONS (Non-Integral):');
const expectedIntegral = parseSection(expectedText, 'CITATIONS (Integral):');

// Compare bibliography
const bibTotal = Math.max(expectedBibEntries.length, actualBibEntries.length);
let bibPassed = 0;
const bibEntries = [];

for (let i = 0; i < bibTotal; i++) {
  const exp = expectedBibEntries[i] ?? '';
  const act = actualBibEntries[i] ?? '';
  const match = exp === act;
  if (match) bibPassed++;
  bibEntries.push({ expected: exp, actual: act, match });
}

// Compare citations (non-integral + integral combined)
const expectedCitations = [...expectedNonIntegral, ...expectedIntegral];
const actualCitations = [...actualNonIntegral, ...actualIntegral];
const citTotal = Math.max(expectedCitations.length, actualCitations.length);
let citPassed = 0;

for (let i = 0; i < citTotal; i++) {
  const exp = expectedCitations[i] ?? '';
  const act = actualCitations[i] ?? '';
  if (exp === act) citPassed++;
}

const perfectMatch = actualText === expectedText;
const citationsByType = buildCitationsByType(bibEntries, setGroups);

emit(
  {
    citations: { passed: citPassed, total: citTotal },
    bibliography: { passed: bibPassed, total: bibTotal, entries: bibEntries },
    citationsByType,
    componentSummary: {},
  },
  perfectMatch ? 0 : 1
);
