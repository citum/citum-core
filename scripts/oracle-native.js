#!/usr/bin/env node
/**
 * oracle-native.js — Snapshot-based oracle for native Citum-created styles.
 *
 * These styles have no CSL equivalent to diff against. Instead we:
 *   1. Run `citum render refs` against a fixed fixture
 *   2. Compare the bibliography section to a stored snapshot
 *   3. In bootstrap mode (snapshot absent) write it and report fidelity 1.0
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

/** Extract non-empty bibliography lines from rendered output */
function parseBibliographyEntries(text) {
  const lines = text.split('\n');
  const bibStart = lines.findIndex((l) => l.trim() === 'BIBLIOGRAPHY:');
  if (bibStart === -1) return [];
  return lines
    .slice(bibStart + 1)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
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
const actualEntries = parseBibliographyEntries(actualText);
const snapshotPath = path.join(snapshotDir, `${styleName}.txt`);

// Bootstrap mode: snapshot absent — write it and report perfect fidelity
if (!fs.existsSync(snapshotPath)) {
  fs.mkdirSync(snapshotDir, { recursive: true });
  fs.writeFileSync(snapshotPath, actualText, 'utf8');
  const n = actualEntries.length;
  emit(
    {
      citations: { passed: 0, total: 0 },
      bibliography: {
        passed: n,
        total: n,
        entries: actualEntries.map((e) => ({ expected: e, actual: e, match: true })),
      },
      citationsByType: {},
      componentSummary: {},
    },
    0
  );
}

// Comparison mode: diff actual vs snapshot
const expectedText = fs.readFileSync(snapshotPath, 'utf8').trim();
const expectedEntries = parseBibliographyEntries(expectedText);

const total = Math.max(expectedEntries.length, actualEntries.length);
let passed = 0;
const entries = [];

for (let i = 0; i < total; i++) {
  const exp = expectedEntries[i] ?? '';
  const act = actualEntries[i] ?? '';
  const match = exp === act;
  if (match) passed++;
  entries.push({ expected: exp, actual: act, match });
}

const perfectMatch = actualText === expectedText;

emit(
  {
    citations: { passed: 0, total: 0 },
    bibliography: { passed, total, entries },
    citationsByType: {},
    componentSummary: {},
  },
  perfectMatch ? 0 : 1
);
