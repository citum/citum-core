#!/usr/bin/env node
/**
 * Batch oracle for migrate-only evaluation.
 *
 * This script forces oracle runs through `citum-migrate` output via
 * `oracle.js --force-migrate`, avoiding false confidence from
 * checked-in `styles/*.yaml`.
 *
 * Usage:
 *   node scripts/oracle-migrate-batch.js --top 10
 *   node scripts/oracle-migrate-batch.js --styles apa,elsevier-harvard
 *   node scripts/oracle-migrate-batch.js --styles apa,ieee --json
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const WORKSPACE_ROOT = path.resolve(__dirname, '..');
const LEGACY_DIR = path.join(WORKSPACE_ROOT, 'styles-legacy');
const ORACLE = path.join(WORKSPACE_ROOT, 'scripts', 'oracle.js');

const TOP_PARENTS = [
  'apa',
  'elsevier-with-titles',
  'elsevier-harvard',
  'elsevier-vancouver',
  'springer-vancouver-brackets',
  'springer-basic-author-date',
  'springer-basic-brackets',
  'springer-socpsych-author-date',
  'american-medical-association',
  'taylor-and-francis-chicago-author-date',
  'springer-mathphys-brackets',
  'multidisciplinary-digital-publishing-institute',
  'ieee',
  'nlm-citation-sequence-superscript',
  'nlm-citation-sequence',
  'karger-journals',
  'institute-of-physics-numeric',
  'thieme-german',
  'mary-ann-liebert-vancouver',
  'biomed-central',
];

function parseArgs(argv) {
  const args = {
    top: 0,
    styles: null,
    json: false,
    out: null,
    migrateTemplateSource: null,
    migrateMinTemplateConfidence: null,
    migrateTemplateDir: null,
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--top' && argv[i + 1]) {
      args.top = Number(argv[++i]);
    } else if (arg === '--styles' && argv[i + 1]) {
      args.styles = argv[++i];
    } else if (arg === '--json') {
      args.json = true;
    } else if (arg === '--out' && argv[i + 1]) {
      args.out = argv[++i];
    } else if (arg === '--migrate-template-source' && argv[i + 1]) {
      args.migrateTemplateSource = argv[++i];
    } else if (arg === '--migrate-min-template-confidence' && argv[i + 1]) {
      args.migrateMinTemplateConfidence = argv[++i];
    } else if (arg === '--migrate-template-dir' && argv[i + 1]) {
      args.migrateTemplateDir = path.resolve(argv[++i]);
    } else if (arg === '-h' || arg === '--help') {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

function printHelp() {
  console.log('Batch migrate-only oracle runner');
  console.log('');
  console.log('Usage:');
  console.log('  node scripts/oracle-migrate-batch.js --top 10');
  console.log('  node scripts/oracle-migrate-batch.js --styles apa,elsevier-harvard');
  console.log('  node scripts/oracle-migrate-batch.js --styles apa,ieee --json');
  console.log('  node scripts/oracle-migrate-batch.js --top 10 --migrate-template-source inferred');
}

function resolveStyles(args) {
  if (args.styles) {
    return args.styles
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);
  }

  if (args.top > 0) {
    return TOP_PARENTS.slice(0, args.top);
  }

  return fs
    .readdirSync(LEGACY_DIR)
    .filter((name) => name.endsWith('.csl'))
    .map((name) => path.basename(name, '.csl'))
    .sort();
}

function runOracleMigrateOnly(styleName, args) {
  const legacyPath = path.join(LEGACY_DIR, `${styleName}.csl`);
  if (!fs.existsSync(legacyPath)) {
    return {
      style: styleName,
      error: 'missing_legacy_style',
      details: `Missing ${legacyPath}`,
    };
  }

  const oracleArgs = [ORACLE, legacyPath, '--json', '--force-migrate'];
  if (args.migrateTemplateSource) {
    oracleArgs.push('--migrate-template-source', args.migrateTemplateSource);
  }
  if (args.migrateMinTemplateConfidence) {
    oracleArgs.push('--migrate-min-template-confidence', args.migrateMinTemplateConfidence);
  }
  if (args.migrateTemplateDir) {
    oracleArgs.push('--migrate-template-dir', args.migrateTemplateDir);
  }

  const proc = spawnSync('node', oracleArgs, {
    cwd: WORKSPACE_ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  if (proc.status !== 0 && proc.status !== 1) {
    return {
      style: styleName,
      error: 'oracle_exec_failed',
      details: (proc.stderr || '').trim() || `exit=${proc.status}`,
    };
  }

  try {
    const parsed = JSON.parse(proc.stdout || '{}');
    if (parsed.error) {
      return {
        style: styleName,
        error: parsed.error,
        details: parsed.reason || null,
      };
    }
    return {
      style: styleName,
      citations: parsed.citations,
      bibliography: parsed.bibliography,
    };
  } catch (err) {
    return {
      style: styleName,
      error: 'oracle_json_parse_failed',
      details: err.message,
    };
  }
}

function aggregate(results, args) {
  const summary = {
    mode: 'migrate-only',
    migrate: {
      templateSource: args.migrateTemplateSource || 'auto',
      minTemplateConfidence: args.migrateMinTemplateConfidence || null,
      templateDir: args.migrateTemplateDir || null,
    },
    totalStyles: results.length,
    successes: 0,
    failures: 0,
    citations: { passed: 0, total: 0 },
    bibliography: { passed: 0, total: 0 },
    styles: results,
  };

  for (const result of results) {
    if (result.error) {
      summary.failures += 1;
      continue;
    }

    summary.successes += 1;
    summary.citations.passed += result.citations?.passed || 0;
    summary.citations.total += result.citations?.total || 0;
    summary.bibliography.passed += result.bibliography?.passed || 0;
    summary.bibliography.total += result.bibliography?.total || 0;
  }

  return summary;
}

function pct(passed, total) {
  if (!total) return '0.0';
  return ((passed / total) * 100).toFixed(1);
}

function main() {
  const args = parseArgs(process.argv);
  const styles = resolveStyles(args);
  const results = [];
  for (const style of styles) {
    results.push(runOracleMigrateOnly(style, args));
  }

  const summary = aggregate(results, args);
  if (args.out) {
    fs.writeFileSync(path.resolve(args.out), `${JSON.stringify(summary, null, 2)}\n`);
  }

  if (args.json) {
    console.log(JSON.stringify(summary, null, 2));
    return;
  }

  console.log(`Mode: ${summary.mode}`);
  console.log(`Styles: ${summary.totalStyles}`);
  console.log(`Successes: ${summary.successes}`);
  console.log(`Failures: ${summary.failures}`);
  console.log(
    `Citations: ${summary.citations.passed}/${summary.citations.total} (${pct(summary.citations.passed, summary.citations.total)}%)`
  );
  console.log(
    `Bibliography: ${summary.bibliography.passed}/${summary.bibliography.total} (${pct(summary.bibliography.passed, summary.bibliography.total)}%)`
  );
}

try {
  main();
} catch (err) {
  console.error(`Error: ${err.message}`);
  process.exit(1);
}
