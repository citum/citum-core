#!/usr/bin/env node
/**
 * Analyze repeated migration gaps from a report-core JSON artifact.
 *
 * Usage:
 *   node scripts/analyze-migration-gaps.js --report /tmp/core-report.json
 *   node scripts/analyze-migration-gaps.js --report /tmp/core-report.json --min-occurrences 2
 */

'use strict';

const fs = require('fs');
const path = require('path');

function parseArgs(argv) {
  const args = {
    report: null,
    minOccurrences: 2,
  };

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--report' && i + 1 < argv.length) {
      args.report = argv[++i];
    } else if (arg === '--min-occurrences' && i + 1 < argv.length) {
      args.minOccurrences = Number(argv[++i]);
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
  if (!Number.isFinite(args.minOccurrences) || args.minOccurrences < 1) {
    throw new Error('--min-occurrences must be a positive number');
  }

  return args;
}

function printHelp() {
  console.log('Analyze repeated migration gaps from report-core JSON');
  console.log('');
  console.log('Usage:');
  console.log('  node scripts/analyze-migration-gaps.js --report /tmp/core-report.json');
  console.log('  node scripts/analyze-migration-gaps.js --report /tmp/core-report.json --min-occurrences 2');
}

function ratioSafe(passed, total) {
  if (!total) return 1;
  return passed / total;
}

function topEntries(map, minOccurrences) {
  return Array.from(map.entries())
    .filter(([, count]) => count >= minOccurrences)
    .sort((a, b) => b[1] - a[1]);
}

function main() {
  const args = parseArgs(process.argv);
  const reportPath = path.resolve(args.report);
  const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));

  const styles = report.styles || [];
  const citationClusters = new Map();
  const bibliographyClusters = new Map();
  const targets = [];

  for (const style of styles) {
    const citationRatio = ratioSafe(style.citations?.passed || 0, style.citations?.total || 0);
    const bibliographyRatio = ratioSafe(
      style.bibliography?.passed || 0,
      style.bibliography?.total || 0
    );
    const fidelity = typeof style.fidelityScore === 'number'
      ? style.fidelityScore
      : (citationRatio + bibliographyRatio) / 2;
    const sqi = typeof style.qualityScore === 'number' ? style.qualityScore : 1;

    if (fidelity < 0.95 || sqi < 0.90) {
      targets.push({
        style: style.name,
        fidelity: Number(fidelity.toFixed(3)),
        sqi: Number(sqi.toFixed(3)),
        citation: Number(citationRatio.toFixed(3)),
        bibliography: Number(bibliographyRatio.toFixed(3)),
      });
    }

    for (const entry of style.citationEntries || []) {
      if (entry.match === false && entry.id) {
        citationClusters.set(entry.id, (citationClusters.get(entry.id) || 0) + 1);
      }
    }

    for (const [cluster, count] of Object.entries(style.componentSummary || {})) {
      bibliographyClusters.set(cluster, (bibliographyClusters.get(cluster) || 0) + count);
    }
  }

  targets.sort((a, b) => (a.fidelity - b.fidelity) || (a.sqi - b.sqi));

  const output = {
    report: reportPath,
    generated: new Date().toISOString(),
    thresholds: {
      fidelity: 0.95,
      sqi: 0.90,
    },
    totals: {
      styles: styles.length,
      targets: targets.length,
      citation: report.citationsOverall || null,
      bibliography: report.bibliographyOverall || null,
      sqi: report.qualityOverall || null,
    },
    targetStyles: targets,
    citationClusters: topEntries(citationClusters, args.minOccurrences).map(([id, count]) => ({
      id,
      count,
    })),
    bibliographyClusters: topEntries(bibliographyClusters, args.minOccurrences).map(
      ([cluster, count]) => ({
        cluster,
        count,
      })
    ),
  };

  process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
}

try {
  main();
} catch (error) {
  console.error(`Error: ${error.message}`);
  process.exit(1);
}
