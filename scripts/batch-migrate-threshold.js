#!/usr/bin/env node
/**
 * Batch migrate styles selected by fidelity/SQI thresholds.
 *
 * Selection default: fidelity < 1.0 OR SQI < 0.85
 *
 * For each target style:
 * 1) Capture baseline oracle metrics
 * 2) Run prep-migration workflow
 * 3) Capture rerun oracle metrics
 * 4) Keep style only if no regression and at least one metric improves
 *
 * Usage:
 *   node scripts/batch-migrate-threshold.js --report /tmp/core-report.json
 *   node scripts/batch-migrate-threshold.js --report /tmp/core-report.json --limit 20
 *   node scripts/batch-migrate-threshold.js --report /tmp/core-report.json --min-sqi 0.85 --min-fidelity 1.0
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const LEGACY_SOURCE_OVERRIDES = {
  'apa-7th': 'apa',
  'din-alphanumeric': 'din-1505-2-alphanumeric',
  'gost-r-7-0-5-2008-author-date': 'gost-r-7-0-5-2008',
};

function parseArgs(argv) {
  const args = {
    report: null,
    minFidelity: 1.0,
    minSqi: 0.85,
    limit: 0,
    out: null,
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--report' && i + 1 < argv.length) {
      args.report = argv[++i];
    } else if (arg === '--min-fidelity' && i + 1 < argv.length) {
      args.minFidelity = Number(argv[++i]);
    } else if (arg === '--min-sqi' && i + 1 < argv.length) {
      args.minSqi = Number(argv[++i]);
    } else if (arg === '--limit' && i + 1 < argv.length) {
      args.limit = Number(argv[++i]);
    } else if (arg === '--out' && i + 1 < argv.length) {
      args.out = argv[++i];
    } else if (arg === '-h' || arg === '--help') {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.report) {
    throw new Error('Missing required --report argument');
  }
  if (!Number.isFinite(args.minFidelity) || args.minFidelity <= 0 || args.minFidelity > 1) {
    throw new Error('--min-fidelity must be in (0, 1]');
  }
  if (!Number.isFinite(args.minSqi) || args.minSqi <= 0 || args.minSqi > 1) {
    throw new Error('--min-sqi must be in (0, 1]');
  }
  if (!Number.isFinite(args.limit) || args.limit < 0) {
    throw new Error('--limit must be >= 0');
  }

  return args;
}

function printHelp() {
  console.log('Batch migrate styles below fidelity/SQI thresholds');
  console.log('');
  console.log('Usage:');
  console.log('  node scripts/batch-migrate-threshold.js --report /tmp/core-report.json');
  console.log('  node scripts/batch-migrate-threshold.js --report /tmp/core-report.json --limit 20');
  console.log('  node scripts/batch-migrate-threshold.js --report /tmp/core-report.json --out /tmp/batch-summary.json');
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    ...options,
  });
  return result;
}

function parseOracleJson(stdout) {
  try {
    return JSON.parse(stdout);
  } catch (error) {
    throw new Error(`failed to parse oracle JSON: ${error.message}`);
  }
}

function runOracle(workspaceRoot, legacyStylePath) {
  const scriptPath = path.join(workspaceRoot, 'scripts', 'oracle.js');
  const result = run('node', [scriptPath, legacyStylePath, '--json'], { cwd: workspaceRoot });

  // oracle.js exits 1 for mismatch output but still prints valid JSON.
  if (result.status === 0 || result.status === 1) {
    return parseOracleJson(result.stdout || '');
  }

  throw new Error(
    `oracle failed (exit=${result.status}): ${(result.stderr || '').trim() || 'unknown error'}`
  );
}

function runPrepMigration(workspaceRoot, legacyStylePath) {
  // Route noisy output to a temp log for post-mortem only on failure.
  const logPath = path.join(workspaceRoot, '.tmp_migration', `batch-${Date.now()}.log`);
  fs.mkdirSync(path.dirname(logPath), { recursive: true });
  const command = `./scripts/prep-migration.sh "${legacyStylePath}" --agent > "${logPath}" 2>&1`;
  const result = run('/bin/bash', ['-lc', command], { cwd: workspaceRoot });
  return { status: result.status, logPath, stderr: result.stderr || '' };
}

function selectTargets(report, minFidelity, minSqi) {
  return (report.styles || [])
    .filter((style) => (style.fidelityScore ?? 1) < minFidelity || (style.qualityScore ?? 1) < minSqi)
    .sort((a, b) => (a.fidelityScore - b.fidelityScore) || (a.qualityScore - b.qualityScore));
}

function main() {
  const args = parseArgs(process.argv);
  const workspaceRoot = path.resolve(__dirname, '..');
  const report = JSON.parse(fs.readFileSync(path.resolve(args.report), 'utf8'));

  let targets = selectTargets(report, args.minFidelity, args.minSqi);
  if (args.limit > 0) {
    targets = targets.slice(0, args.limit);
  }

  const summary = {
    generated: new Date().toISOString(),
    thresholds: {
      fidelity: args.minFidelity,
      sqi: args.minSqi,
    },
    totalTargets: targets.length,
    accepted: [],
    rejected: [],
    skipped: [],
    failed: [],
  };

  console.log(`Selected ${targets.length} target styles.`);

  targets.forEach((style, idx) => {
    const name = style.name;
    const legacyName = LEGACY_SOURCE_OVERRIDES[name] || name;
    const legacyStylePath = path.join('styles-legacy', `${legacyName}.csl`);
    const stylePath = path.join(workspaceRoot, 'styles', `${name}.yaml`);

    process.stdout.write(`[${idx + 1}/${targets.length}] ${name} ... `);

    if (!fs.existsSync(path.join(workspaceRoot, legacyStylePath))) {
      console.log('SKIP (legacy style missing)');
      summary.skipped.push({ style: name, reason: 'missing legacy style', legacyStylePath });
      return;
    }
    if (!fs.existsSync(stylePath)) {
      console.log('SKIP (target style missing)');
      summary.skipped.push({ style: name, reason: 'missing styles yaml', stylePath });
      return;
    }

    const originalYaml = fs.readFileSync(stylePath, 'utf8');

    let before;
    try {
      before = runOracle(workspaceRoot, legacyStylePath);
    } catch (error) {
      console.log(`FAIL (baseline oracle: ${error.message})`);
      summary.failed.push({ style: name, step: 'baseline-oracle', error: error.message });
      return;
    }

    const prep = runPrepMigration(workspaceRoot, legacyStylePath);
    if (prep.status !== 0) {
      fs.writeFileSync(stylePath, originalYaml);
      console.log('FAIL (prep-migration)');
      summary.failed.push({
        style: name,
        step: 'prep-migration',
        logPath: prep.logPath,
      });
      return;
    }

    let after;
    try {
      after = runOracle(workspaceRoot, legacyStylePath);
    } catch (error) {
      fs.writeFileSync(stylePath, originalYaml);
      console.log(`FAIL (post oracle: ${error.message})`);
      summary.failed.push({ style: name, step: 'post-oracle', error: error.message });
      return;
    }

    const beforeCit = before.citations?.passed || 0;
    const beforeBib = before.bibliography?.passed || 0;
    const afterCit = after.citations?.passed || 0;
    const afterBib = after.bibliography?.passed || 0;

    const noRegression = afterCit >= beforeCit && afterBib >= beforeBib;
    const improved = afterCit > beforeCit || afterBib > beforeBib;

    if (noRegression && improved) {
      console.log(`KEEP (cit ${beforeCit}->${afterCit}, bib ${beforeBib}->${afterBib})`);
      summary.accepted.push({
        style: name,
        legacy: legacyName,
        citations: { before: beforeCit, after: afterCit },
        bibliography: { before: beforeBib, after: afterBib },
      });
      return;
    }

    fs.writeFileSync(stylePath, originalYaml);
    console.log(`REVERT (cit ${beforeCit}->${afterCit}, bib ${beforeBib}->${afterBib})`);
    summary.rejected.push({
      style: name,
      legacy: legacyName,
      citations: { before: beforeCit, after: afterCit },
      bibliography: { before: beforeBib, after: afterBib },
      reason: noRegression ? 'no improvement' : 'regression',
    });
  });

  const summaryLine = `accepted=${summary.accepted.length} rejected=${summary.rejected.length} skipped=${summary.skipped.length} failed=${summary.failed.length}`;
  console.log(`Done: ${summaryLine}`);

  if (args.out) {
    fs.writeFileSync(path.resolve(args.out), `${JSON.stringify(summary, null, 2)}\n`);
    console.log(`Wrote summary: ${path.resolve(args.out)}`);
  } else {
    console.log(JSON.stringify(summary, null, 2));
  }
}

try {
  main();
} catch (error) {
  console.error(`Error: ${error.message}`);
  process.exit(1);
}
