#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function usage() {
  console.error('Usage: node scripts/check-oracle-regression.js --baseline <path> [--styles-dir <path>]');
}

function parseArgs(argv) {
  const args = {
    baseline: null,
    stylesDir: 'styles-legacy',
  };

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--baseline') {
      args.baseline = argv[++i] || null;
    } else if (arg === '--styles-dir') {
      args.stylesDir = argv[++i] || 'styles-legacy';
    }
  }

  return args;
}

function parseScore(score, label) {
  const match = /^(\d+)\/(\d+)$/.exec(String(score || ''));
  if (!match) {
    throw new Error(`Invalid ${label} score format: ${score}`);
  }
  return {
    passed: Number(match[1]),
    total: Number(match[2]),
  };
}

function styleMapFromBreakdown(styleBreakdown) {
  const map = new Map();
  for (const style of styleBreakdown || []) {
    map.set(style.style, style);
  }
  return map;
}

function runAggregate(stylesDir, styles) {
  const scriptPath = path.resolve(__dirname, 'oracle-batch-aggregate.js');
  const styleArg = styles.join(',');
  const cmd = `node "${scriptPath}" "${stylesDir}" --styles "${styleArg}" --json`;
  const output = execSync(cmd, {
    cwd: path.resolve(__dirname, '..'),
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
    maxBuffer: 10 * 1024 * 1024,
  });

  return JSON.parse(output);
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (!args.baseline) {
    usage();
    process.exit(2);
  }

  const baselinePath = path.resolve(args.baseline);
  if (!fs.existsSync(baselinePath)) {
    console.error(`Baseline file not found: ${baselinePath}`);
    process.exit(2);
  }

  const baseline = JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
  const baselineBreakdown = baseline.styleBreakdown;
  if (!Array.isArray(baselineBreakdown) || baselineBreakdown.length === 0) {
    console.error('Baseline file is missing styleBreakdown entries.');
    process.exit(2);
  }

  const baselineStyles = Array.isArray(baseline.metadata?.styles)
    ? baseline.metadata.styles
    : baselineBreakdown.map((s) => s.style);

  if (!baselineStyles.length) {
    console.error('Baseline does not define any styles to check.');
    process.exit(2);
  }

  console.log(`Running oracle regression check for ${baselineStyles.length} styles...`);
  const current = runAggregate(args.stylesDir, baselineStyles);

  const baselineMap = styleMapFromBreakdown(baselineBreakdown);
  const currentMap = styleMapFromBreakdown(current.styleBreakdown);

  const regressions = [];
  const improvements = [];

  for (const styleName of baselineStyles) {
    const baselineStyle = baselineMap.get(styleName);
    const currentStyle = currentMap.get(styleName);

    if (!baselineStyle) {
      regressions.push({
        style: styleName,
        reason: 'Style missing from baseline styleBreakdown',
      });
      continue;
    }

    if (!currentStyle) {
      regressions.push({
        style: styleName,
        reason: 'Style missing from current aggregate output',
      });
      continue;
    }

    const baselineCit = parseScore(baselineStyle.citations, `${styleName} citations (baseline)`);
    const baselineBib = parseScore(baselineStyle.bibliography, `${styleName} bibliography (baseline)`);
    const currentCit = parseScore(currentStyle.citations, `${styleName} citations (current)`);
    const currentBib = parseScore(currentStyle.bibliography, `${styleName} bibliography (current)`);

    if (baselineCit.total !== currentCit.total || baselineBib.total !== currentBib.total) {
      regressions.push({
        style: styleName,
        reason: `Fixture totals changed (C ${baselineCit.total}->${currentCit.total}, B ${baselineBib.total}->${currentBib.total})`,
      });
      continue;
    }

    const citationsDelta = currentCit.passed - baselineCit.passed;
    const bibliographyDelta = currentBib.passed - baselineBib.passed;

    if (citationsDelta < 0 || bibliographyDelta < 0) {
      regressions.push({
        style: styleName,
        reason: [
          citationsDelta < 0 ? `citations ${baselineStyle.citations} -> ${currentStyle.citations}` : null,
          bibliographyDelta < 0 ? `bibliography ${baselineStyle.bibliography} -> ${currentStyle.bibliography}` : null,
        ].filter(Boolean).join(', '),
      });
    } else if (citationsDelta > 0 || bibliographyDelta > 0) {
      improvements.push({
        style: styleName,
        reason: [
          citationsDelta > 0 ? `citations ${baselineStyle.citations} -> ${currentStyle.citations}` : null,
          bibliographyDelta > 0 ? `bibliography ${baselineStyle.bibliography} -> ${currentStyle.bibliography}` : null,
        ].filter(Boolean).join(', '),
      });
    }
  }

  if (regressions.length > 0) {
    console.error('\nOracle regressions detected:');
    for (const regression of regressions) {
      console.error(`- ${regression.style}: ${regression.reason}`);
    }
    process.exit(1);
  }

  console.log('No oracle regressions detected.');
  if (improvements.length > 0) {
    console.log(`Improvements detected in ${improvements.length} style(s):`);
    for (const improvement of improvements) {
      console.log(`- ${improvement.style}: ${improvement.reason}`);
    }
  }
}

try {
  main();
} catch (error) {
  console.error(`check-oracle-regression failed: ${error.message}`);
  process.exit(1);
}
