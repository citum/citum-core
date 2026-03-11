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
  console.log(`Pass: ${result.summary.pass}`);
  console.log(`Configuration gaps: ${result.summary.configurationGap}`);
  console.log(`Rendering gaps: ${result.summary.renderingGap}`);
  console.log(`Missing expectations: ${result.summary.missingExpectations}`);
  console.log(`Extra expectations: ${result.summary.extraExpectations}`);
  console.log('');
  for (const entry of result.results) {
    const issueSummary = entry.issues.length === 0
      ? 'ok'
      : entry.issues.map((issue) => issue.message).join(' | ');
    console.log(`${entry.style}\t${entry.status}\t${entry.profile}\t${issueSummary}`);
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
  const hasFailures = result.results.some((entry) => entry.status !== 'pass');
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
