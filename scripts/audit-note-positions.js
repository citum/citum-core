#!/usr/bin/env node
'use strict';

const { auditAllNoteStyles } = require('./lib/note-position-audit');

function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    jsonOutput: false,
    styles: [],
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--json') {
      options.jsonOutput = true;
    } else if (arg === '--styles') {
      const stylesArg = argv[++i];
      if (!stylesArg || stylesArg.startsWith('--')) {
        throw new Error(
          'Error: --styles option requires a comma-separated list of styles.\n'
          + 'Usage: scripts/audit-note-positions.js [--json] [--styles style1,style2,...]'
        );
      }
      options.styles = stylesArg.split(',').map((value) => value.trim()).filter(Boolean);
    }
  }
  return options;
}

function printHuman(result) {
  console.log('=== Note Position Audit ===');
  console.log(`Styles: ${result.summary.total}`);
  console.log(`Regression pass: ${result.summary.regression.pass}`);
  console.log(`Regression configuration gaps: ${result.summary.regression.configurationGap}`);
  console.log(`Regression rendering gaps: ${result.summary.regression.renderingGap}`);
  console.log(`Conformance pass: ${result.summary.conformance.pass}`);
  console.log(`Conformance gaps: ${result.summary.conformance.gap}`);
  console.log(`Conformance unresolved: ${result.summary.conformance.unresolved}`);
  console.log(`Missing expectations: ${result.summary.missingExpectations}`);
  console.log(`Extra expectations: ${result.summary.extraExpectations}`);
  console.log('');
  for (const entry of result.results) {
    const regressionSummary = entry.regression.issues.length === 0
      ? 'ok'
      : entry.regression.issues.map((issue) => issue.message).join(' | ');
    const conformanceSummary = entry.conformance.issues.length === 0
      ? (entry.conformance.unresolved.length === 0 ? 'ok' : `unresolved: ${entry.conformance.unresolved.join(', ')}`)
      : entry.conformance.issues.map((issue) => issue.message).join(' | ');
    console.log(
      `${entry.style}\tregression=${entry.regression.status}(${entry.regression.profile})\t`
      + `conformance=${entry.conformance.status}(${entry.conformance.family})\t`
      + `regression:${regressionSummary}\tconformance:${conformanceSummary}`
    );
  }
}

function main() {
  const options = parseArgs();
  const result = auditAllNoteStyles({ styles: options.styles });

  if (options.jsonOutput) {
    console.log(JSON.stringify(result, null, 2));
  } else {
    printHuman(result);
  }

  const hasCoverageIssue = result.coverage.missing.length > 0 || result.coverage.extra.length > 0;
  const hasFailures = result.results.some((entry) => entry.regression.status !== 'pass');
  process.exit(hasCoverageIssue || hasFailures ? 1 : 0);
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}

module.exports = {
  parseArgs,
};
