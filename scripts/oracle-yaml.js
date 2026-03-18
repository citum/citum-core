#!/usr/bin/env node
/**
 * Oracle test for hand-authored Citum styles.
 *
 * Known project styles are compared through the structured oracle path so
 * fixture-family resolution matches report-core. Arbitrary YAML paths fall
 * back to a direct citeproc-vs-Citum comparison.
 *
 * Usage:
 *   node oracle-yaml.js styles/apa-7th.yaml --json
 *   node oracle-yaml.js styles/chicago-notes.yaml --fixture-family note-humanities
 *   node oracle-yaml.js /tmp/custom-style.yaml --legacy-csl styles-legacy/apa.csl
 */

'use strict';

const CSL = require('citeproc');
const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');
const yaml = require('js-yaml');
const {
  compareText,
  normalizeText,
  loadLocale,
} = require('./oracle-utils');
const { toCiteprocItem } = require('./lib/citeproc-locators');
const {
  PROJECT_ROOT,
  resolveYamlVerificationPlan,
  resolveStyleData,
} = require('./lib/style-verification');

const CUSTOM_TAG_SCHEMA = yaml.DEFAULT_SCHEMA.extend([
  new yaml.Type('!custom', {
    kind: 'mapping',
    construct(data) {
      return data || {};
    },
  }),
]);

function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    yamlPath: null,
    legacyCslPath: null,
    jsonOutput: false,
    verbose: false,
    caseSensitive: true,
    refsFixture: null,
    citationsFixture: null,
    fixtureFamily: null,
  };

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--json') {
      options.jsonOutput = true;
    } else if (arg === '--verbose') {
      options.verbose = true;
    } else if (arg === '--case-sensitive') {
      options.caseSensitive = true;
    } else if (arg === '--case-insensitive') {
      options.caseSensitive = false;
    } else if (arg === '--legacy-csl') {
      options.legacyCslPath = argv[++i];
    } else if (arg === '--refs-fixture') {
      options.refsFixture = argv[++i];
    } else if (arg === '--citations-fixture') {
      options.citationsFixture = argv[++i];
    } else if (arg === '--fixture-family') {
      options.fixtureFamily = argv[++i];
    } else if (!arg.startsWith('--') && !options.yamlPath) {
      options.yamlPath = arg;
    } else if (!arg.startsWith('--') && !options.legacyCslPath) {
      options.legacyCslPath = arg;
    }
  }

  if (!options.yamlPath) {
    throw new Error(
      'Usage: node oracle-yaml.js <yaml-path> [reference.csl] [--json] [--verbose] ' +
      '[--legacy-csl path] [--fixture-family family] [--refs-fixture path] [--citations-fixture path]'
    );
  }

  return options;
}

function loadStyleData(yamlPath) {
  const rawStyleData = yaml.load(fs.readFileSync(yamlPath, 'utf8'), { schema: CUSTOM_TAG_SCHEMA }) || {};
  return {
    rawStyleData,
    resolvedStyleData: resolveStyleData(rawStyleData),
  };
}

function inferStyleFormat(styleData) {
  const processing = styleData?.options?.processing;
  if (typeof processing === 'string') {
    return processing;
  }
  if (processing && typeof processing === 'object') {
    if (Object.prototype.hasOwnProperty.call(processing, 'note')) return 'note';
    if (Object.prototype.hasOwnProperty.call(processing, 'author-date')) return 'author-date';
    if (
      Object.prototype.hasOwnProperty.call(processing, 'label') ||
      Object.prototype.hasOwnProperty.call(processing, 'numeric')
    ) {
      return 'numeric';
    }
  }
  return 'unknown';
}

function hasBibliographyTemplate(styleData) {
  const bibliography = styleData?.bibliography;
  if (!bibliography || typeof bibliography !== 'object') {
    return false;
  }
  const hasTemplate = Array.isArray(bibliography.template) && bibliography.template.length > 0;
  const hasTypeTemplates = Boolean(
    bibliography['type-templates'] && Object.keys(bibliography['type-templates']).length > 0
  );
  return hasTemplate || hasTypeTemplates;
}

function runOracleScript(cslPath, refsFixture, citationsFixture, options = {}) {
  const scriptPath = path.join(__dirname, 'oracle.js');
  try {
    const stdout = execFileSync(
      process.execPath,
      [
        scriptPath,
        cslPath,
        '--json',
        '--refs-fixture',
        refsFixture,
        '--citations-fixture',
        citationsFixture,
        ...(options.caseSensitive === false ? ['--case-insensitive'] : ['--case-sensitive']),
      ],
      {
        cwd: PROJECT_ROOT,
        encoding: 'utf8',
        stdio: ['pipe', 'pipe', 'pipe'],
      }
    );
    return JSON.parse(stdout);
  } catch (error) {
    if (typeof error.stdout === 'string' && error.stdout.trim()) {
      return JSON.parse(error.stdout);
    }
    throw error;
  }
}

function mergeStructuredResults(main, extra) {
  if (!extra) return main;

  main.citations = {
    total: (main.citations?.total || 0) + (extra.citations?.total || 0),
    passed: (main.citations?.passed || 0) + (extra.citations?.passed || 0),
    failed: (main.citations?.failed || 0) + (extra.citations?.failed || 0),
    entries: [...(main.citations?.entries || []), ...(extra.citations?.entries || [])],
  };
  main.bibliography = {
    total: (main.bibliography?.total || 0) + (extra.bibliography?.total || 0),
    passed: (main.bibliography?.passed || 0) + (extra.bibliography?.passed || 0),
    failed: (main.bibliography?.failed || 0) + (extra.bibliography?.failed || 0),
    entries: [...(main.bibliography?.entries || []), ...(extra.bibliography?.entries || [])],
  };

  const mergedTypes = { ...(main.citationsByType || {}) };
  for (const [typeName, stats] of Object.entries(extra.citationsByType || {})) {
    const current = mergedTypes[typeName] || { total: 0, passed: 0 };
    mergedTypes[typeName] = {
      total: current.total + (stats.total || 0),
      passed: current.passed + (stats.passed || 0),
    };
  }
  main.citationsByType = mergedTypes;
  const mergedComponents = { ...(main.componentSummary || {}) };
  for (const [componentName, count] of Object.entries(extra.componentSummary || {})) {
    const currentCount = mergedComponents[componentName] || 0;
    mergedComponents[componentName] = currentCount + (count || 0);
  }
  main.componentSummary = mergedComponents;
  main.orderingIssues = (main.orderingIssues || 0) + (extra.orderingIssues || 0);

  return main;
}

function shouldUseStructuredOracle(options, stylePlan) {
  if (!stylePlan.canUseStructuredOracle) {
    return false;
  }

  // Explicit CSL overrides should validate the exact YAML path requested, not
  // whichever YAML would be inferred from the CSL basename inside oracle.js.
  if (options.legacyCslPath) {
    return false;
  }

  return true;
}

function normalizeFixtureItems(fixturesData) {
  if (Array.isArray(fixturesData)) {
    return Object.fromEntries(fixturesData.map((item) => [item.id, item]));
  }
  if (fixturesData && Array.isArray(fixturesData.references)) {
    return Object.fromEntries(fixturesData.references.map((item) => [item.id, item]));
  }
  return Object.fromEntries(
    Object.entries(fixturesData).filter(([key, value]) => key !== 'comment' && value && typeof value === 'object')
  );
}

function loadFixtures(refsFixture, citationsFixture) {
  const refsData = JSON.parse(fs.readFileSync(refsFixture, 'utf8'));
  const testItems = normalizeFixtureItems(refsData);
  const testCitations = JSON.parse(fs.readFileSync(citationsFixture, 'utf8'));
  return { refsData, testItems, testCitations };
}

function renderWithCslnYaml(yamlPath, refsFixture, citationsFixture) {
  const absYamlPath = path.resolve(yamlPath);
  const output = execFileSync(
    'cargo',
    [
      'run',
      '-q',
      '--bin',
      'citum',
      '--',
      'render',
      'refs',
      '-b',
      refsFixture,
      '-s',
      absYamlPath,
      '-c',
      citationsFixture,
      '--mode',
      'both',
      '--show-keys',
    ],
    {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    }
  );

  const lines = output.split('\n');
  const citations = {};
  const bibliography = [];
  let section = null;

  for (const line of lines) {
    if (line.includes('CITATIONS')) {
      section = 'citations';
    } else if (line.includes('BIBLIOGRAPHY:')) {
      section = 'bibliography';
    } else if (section === 'citations' && line.match(/\[[^\]]+\]/)) {
      const match = line.match(/\[([^\]]+)\]\s*(.+)/);
      if (match && !citations[match[1]]) {
        citations[match[1]] = match[2].trim();
      }
    } else if (section === 'bibliography' && line.trim() && !line.includes('===')) {
      const match = line.match(/\[([^\]]+)\]\s+(.+)/);
      bibliography.push(match ? match[2].trim() : line.trim());
    }
  }

  return { citations, bibliography };
}

function renderWithCiteprocJs(cslPath, refsFixture, citationsFixture) {
  const { testItems, testCitations } = loadFixtures(refsFixture, citationsFixture);
  const styleXml = fs.readFileSync(cslPath, 'utf8');
  const sys = {
    retrieveLocale: (lang) => loadLocale(lang),
    retrieveItem: (id) => testItems[id],
  };

  const citeproc = new CSL.Engine(sys, styleXml);
  citeproc.updateItems(Object.keys(testItems));

  const citations = {};
  for (const citation of testCitations) {
    const suppressAuthor = citation['suppress-author'] === true;
    const citeprocItems = citation.items.map((item) => toCiteprocItem(item, suppressAuthor));
    citations[citation.id] = citeproc.makeCitationCluster(citeprocItems);
  }

  const bibResult = citeproc.makeBibliography();
  return { citations, bibliography: bibResult ? bibResult[1] : [] };
}

function matchBibliographyEntries(oracleBib, cslnBib) {
  const pairs = [];
  const usedCsln = new Set();

  for (const oracleEntry of oracleBib) {
    const oracleNorm = normalizeText(oracleEntry).toLowerCase();
    let bestMatch = null;
    let bestScore = 0;

    for (let i = 0; i < cslnBib.length; i++) {
      if (usedCsln.has(i)) continue;
      const cslnNorm = normalizeText(cslnBib[i]).toLowerCase();
      const oracleWords = new Set(oracleNorm.split(/\s+/).filter((word) => word.length > 3));
      const cslnWords = new Set(cslnNorm.split(/\s+/).filter((word) => word.length > 3));
      let score = 0;
      for (const word of oracleWords) {
        if (cslnWords.has(word)) score += 1;
      }
      if (score > bestScore) {
        bestScore = score;
        bestMatch = i;
      }
    }

    if (bestMatch !== null && bestScore > 2) {
      pairs.push({ oracle: oracleEntry, csln: cslnBib[bestMatch], score: bestScore });
      usedCsln.add(bestMatch);
    } else {
      pairs.push({ oracle: oracleEntry, csln: null, score: 0 });
    }
  }

  for (let i = 0; i < cslnBib.length; i++) {
    if (!usedCsln.has(i)) {
      pairs.push({ oracle: null, csln: cslnBib[i], score: 0 });
    }
  }

  return pairs;
}

function runDirectComparison(options, stylePlan) {
  const { refsFixture, citationsFixture } = stylePlan.baseRun;
  const cslnResult = renderWithCslnYaml(options.yamlPath, refsFixture, citationsFixture);
  const citeprocResult = renderWithCiteprocJs(stylePlan.legacyCslPath, refsFixture, citationsFixture);
  const citationResults = [];
  const bibResults = [];

  for (const [citationId, cslnCitation] of Object.entries(cslnResult.citations)) {
    const oracleCitation = citeprocResult.citations[citationId];
    if (!oracleCitation) continue;
    const comparison = compareText(oracleCitation, cslnCitation, {
      caseSensitive: options.caseSensitive,
    });
    citationResults.push({
      itemId: citationId,
      oracle: comparison.expected,
      csln: comparison.actual,
      match: comparison.match,
      caseMismatch: comparison.caseMismatch,
    });
  }

  for (const pair of matchBibliographyEntries(citeprocResult.bibliography, cslnResult.bibliography)) {
    const comparison = pair.oracle && pair.csln
      ? compareText(pair.oracle, pair.csln, { caseSensitive: options.caseSensitive })
      : null;
    bibResults.push({
      oracle: comparison ? comparison.expected : pair.oracle,
      csln: comparison ? comparison.actual : pair.csln,
      match: Boolean(comparison?.match),
      caseMismatch: Boolean(comparison?.caseMismatch),
      score: pair.score,
    });
  }

  return {
    style: stylePlan.styleName,
    citations: {
      total: citationResults.length,
      passed: citationResults.filter((entry) => entry.match).length,
      failed: citationResults.filter((entry) => !entry.match).length,
      entries: citationResults.map((entry) => ({
        id: entry.itemId,
        oracle: entry.oracle,
        csln: entry.csln,
        match: entry.match,
      })),
    },
    bibliography: {
      total: bibResults.length,
      passed: bibResults.filter((entry) => entry.match).length,
      failed: bibResults.filter((entry) => !entry.match).length,
      entries: bibResults,
    },
  };
}

function runComparison(options) {
  const { rawStyleData, resolvedStyleData } = loadStyleData(options.yamlPath);
  const stylePlan = resolveYamlVerificationPlan({
    yamlPath: options.yamlPath,
    legacyCslPath: options.legacyCslPath,
    refsFixture: options.refsFixture,
    citationsFixture: options.citationsFixture,
    fixtureFamily: options.fixtureFamily,
    styleData: rawStyleData,
    resolvedStyleData,
    styleFormat: inferStyleFormat(resolvedStyleData),
    hasBibliography: hasBibliographyTemplate(resolvedStyleData),
  });

  if (shouldUseStructuredOracle(options, stylePlan)) {
    const result = runOracleScript(
      stylePlan.legacyCslPath,
      stylePlan.baseRun.refsFixture,
      stylePlan.baseRun.citationsFixture,
      options
    );
    for (const familyRun of stylePlan.familyRuns) {
      const extra = runOracleScript(
        stylePlan.legacyCslPath,
        familyRun.refsFixture,
        familyRun.citationsFixture,
        options
      );
      mergeStructuredResults(result, extra);
    }
    return result;
  }

  return runDirectComparison(options, stylePlan);
}

function summarizeResults(results) {
  return {
    style: results.style,
    citations: {
      total: results.citations?.total || 0,
      matches: results.citations?.passed || 0,
      mismatches: results.citations?.failed || 0,
    },
    bibliography: {
      total: results.bibliography?.total || 0,
      matches: results.bibliography?.passed || 0,
      mismatches: results.bibliography?.failed || 0,
    },
    entries: {
      citations: (results.citations?.entries || []).map((entry) => ({
        itemId: entry.id || entry.itemId,
        oracle: entry.oracle,
        csln: entry.csln,
        match: entry.match,
      })),
      bibliography: results.bibliography?.entries || [],
    },
  };
}

function renderText(summary, verbose) {
  let output = `\nOracle Test: ${summary.style}\n`;
  output += `${'='.repeat(50)}\n\n`;
  output += `CITATIONS: ${summary.citations.matches}/${summary.citations.total} match\n`;

  if (verbose) {
    const failedCitations = summary.entries.citations.filter((entry) => !entry.match).slice(0, 3);
    for (const entry of failedCitations) {
      output += `  ✗ ${entry.itemId}\n`;
      output += `    Expected: ${String(entry.oracle).substring(0, 60)}...\n`;
      output += `    Got:      ${String(entry.csln).substring(0, 60)}...\n`;
    }
  }

  output += '\n';
  output += `BIBLIOGRAPHY: ${summary.bibliography.matches}/${summary.bibliography.total} match\n`;
  if (verbose) {
    const failedBibliography = summary.entries.bibliography.filter((entry) => !entry.match).slice(0, 3);
    for (const entry of failedBibliography) {
      output += '  ✗ Entry\n';
      output += `    Expected: ${String(entry.oracle).substring(0, 60)}...\n`;
      output += `    Got:      ${entry.csln ? String(entry.csln).substring(0, 60) : '(missing)'}\n`;
    }
  }

  output += `\n${'='.repeat(50)}\n`;
  return output;
}

function main() {
  try {
    const options = parseArgs();
    const results = runComparison(options);
    const summary = summarizeResults(results);
    if (options.jsonOutput) {
      console.log(JSON.stringify(summary, null, 2));
    } else {
      console.log(renderText(summary, options.verbose));
    }

    const hasFailures = summary.citations.mismatches > 0 || summary.bibliography.mismatches > 0;
    process.exitCode = hasFailures ? 1 : 0;
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 2;
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  hasBibliographyTemplate,
  inferStyleFormat,
  mergeStructuredResults,
  parseArgs,
  resolveYamlVerificationPlan,
  runComparison,
  shouldUseStructuredOracle,
  summarizeResults,
};
