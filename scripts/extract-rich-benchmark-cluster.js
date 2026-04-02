#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const CSL = require('citeproc');
const yaml = require('js-yaml');

const {
  loadVerificationPolicy,
  resolveVerificationPolicy,
} = require('./lib/verification-policy');
const {
  LEGACY_STYLES_DIR,
  STYLES_DIR,
  inferLegacySourceName,
  PROJECT_ROOT,
} = require('./lib/style-verification');
const {
  loadLocale,
  normalizeText,
} = require('./oracle-utils');

const ORACLE_SCRIPT = path.join(__dirname, 'oracle.js');

function consumeFlagValue(args, index, flag) {
  const value = args[index + 1];
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error(`Missing value for ${flag}`);
  }
  return {
    value,
    nextIndex: index + 1,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    style: null,
    benchmark: null,
    types: null,
    ids: null,
    onlyMismatches: false,
    outDir: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--style') {
      const consumed = consumeFlagValue(argv, index, '--style');
      options.style = consumed.value.trim();
      index = consumed.nextIndex;
    } else if (arg === '--benchmark') {
      const consumed = consumeFlagValue(argv, index, '--benchmark');
      options.benchmark = consumed.value.trim();
      index = consumed.nextIndex;
    } else if (arg === '--type') {
      const consumed = consumeFlagValue(argv, index, '--type');
      options.types = consumed.value
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean);
      index = consumed.nextIndex;
    } else if (arg === '--ids') {
      const consumed = consumeFlagValue(argv, index, '--ids');
      options.ids = consumed.value
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean);
      index = consumed.nextIndex;
    } else if (arg === '--only-mismatches') {
      options.onlyMismatches = true;
    } else if (arg === '--out-dir') {
      const consumed = consumeFlagValue(argv, index, '--out-dir');
      options.outDir = path.resolve(consumed.value);
      index = consumed.nextIndex;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!options.style) {
    throw new Error('Missing required flag: --style');
  }
  if (!options.outDir) {
    throw new Error('Missing required flag: --out-dir');
  }
  if ((!options.types || options.types.length === 0) && (!options.ids || options.ids.length === 0)) {
    throw new Error('One selector is required: --type or --ids');
  }
  if (options.types?.length && options.ids?.length) {
    throw new Error('Flags --type and --ids are mutually exclusive');
  }

  return options;
}

function loadStyleData(styleName) {
  const stylePath = path.join(STYLES_DIR, `${styleName}.yaml`);
  if (!fs.existsSync(stylePath)) {
    throw new Error(`Style YAML not found: ${stylePath}`);
  }
  return {
    stylePath,
    styleData: yaml.load(fs.readFileSync(stylePath, 'utf8')) || {},
  };
}

function resolveLegacyStylePath(styleName, styleData) {
  const sourceName = inferLegacySourceName(styleName, styleData);
  const legacyStylePath = path.join(LEGACY_STYLES_DIR, `${sourceName}.csl`);
  if (!fs.existsSync(legacyStylePath)) {
    throw new Error(`Legacy CSL not found for ${styleName}: ${legacyStylePath}`);
  }
  return legacyStylePath;
}

function resolveBenchmarkRun(styleName, benchmarkId, policy = loadVerificationPolicy()) {
  const stylePolicy = resolveVerificationPolicy(styleName, policy);
  const benchmarkRuns = stylePolicy.benchmarkRuns || [];

  if (benchmarkRuns.length === 0) {
    throw new Error(`No benchmark_runs configured for style: ${styleName}`);
  }

  let benchmarkRun;
  if (benchmarkId) {
    benchmarkRun = benchmarkRuns.find((run) => run.id === benchmarkId);
    if (!benchmarkRun) {
      throw new Error(`Benchmark run not found for ${styleName}: ${benchmarkId}`);
    }
  } else {
    benchmarkRun = benchmarkRuns.find((run) => run.countTowardFidelity && run.runner === 'citeproc-oracle')
      || benchmarkRuns.find((run) => run.runner === 'citeproc-oracle');
  }

  if (!benchmarkRun) {
    throw new Error(`No citeproc-oracle benchmark run configured for style: ${styleName}`);
  }
  if (benchmarkRun.runner !== 'citeproc-oracle') {
    throw new Error(`Benchmark run ${benchmarkRun.id} uses unsupported runner: ${benchmarkRun.runner}`);
  }
  if (benchmarkRun.scope !== 'bibliography') {
    throw new Error(`Benchmark run ${benchmarkRun.id} must use bibliography scope in v1`);
  }

  return {
    ...benchmarkRun,
    refsFixture: path.resolve(PROJECT_ROOT, benchmarkRun.refsFixture),
    citationsFixture: benchmarkRun.citationsFixture
      ? path.resolve(PROJECT_ROOT, benchmarkRun.citationsFixture)
      : null,
  };
}

function normalizeFixtureItems(fixturesData) {
  if (Array.isArray(fixturesData)) {
    return fixturesData;
  }
  if (fixturesData && Array.isArray(fixturesData.items)) {
    return fixturesData.items;
  }
  if (fixturesData && Array.isArray(fixturesData.references)) {
    return fixturesData.references;
  }
  return Object.entries(fixturesData || {})
    .filter(([key, value]) => key !== 'comment' && value && typeof value === 'object')
    .map(([, value]) => value);
}

function loadFixtureItems(filePath) {
  const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  return normalizeFixtureItems(data);
}

function createFixtureObject(items) {
  return { items };
}

function selectFixtureItems(items, selectors) {
  if (selectors.types?.length) {
    const allowed = new Set(selectors.types);
    return items.filter((item) => allowed.has(item.type));
  }
  if (selectors.ids?.length) {
    const allowed = new Set(selectors.ids);
    return items.filter((item) => allowed.has(item.id));
  }
  return [];
}

function makeTempFixture(items) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-rich-cluster-'));
  const fixturePath = path.join(dir, 'fixture.json');
  fs.writeFileSync(fixturePath, JSON.stringify(createFixtureObject(items), null, 2));
  return { dir, fixturePath };
}

function cleanupTempFixture(tempFixture) {
  if (!tempFixture?.dir) return;
  fs.rmSync(tempFixture.dir, { recursive: true, force: true });
}

function runOracleBibliography(stylePath, refsFixture) {
  const result = spawnSync(
    process.execPath,
    [
      ORACLE_SCRIPT,
      stylePath,
      '--json',
      '--scope', 'bibliography',
      '--refs-fixture', refsFixture,
    ],
    {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
    }
  );

  if (result.status !== 0 && result.status !== 1) {
    throw new Error(
      `oracle.js failed (${result.status ?? 'signal'}): ${result.stderr || result.stdout || 'no output'}`
    );
  }

  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`oracle.js returned invalid JSON: ${error.message}`);
  }
}

function renderOracleBibliographyRows(stylePath, items) {
  const styleXml = fs.readFileSync(stylePath, 'utf8');
  const itemMap = Object.fromEntries(items.map((item) => [item.id, item]));
  const sys = {
    retrieveLocale: (lang) => loadLocale(lang),
    retrieveItem: (id) => itemMap[id],
  };

  const citeproc = new CSL.Engine(sys, styleXml);
  citeproc.updateItems(items.map((item) => item.id));

  const bibliography = citeproc.makeBibliography();
  if (!bibliography) {
    return [];
  }

  const entryIds = bibliography[0]?.entry_ids || [];
  const entries = bibliography[1] || [];
  return entries.map((entry, index) => ({
    id: entryIds[index]?.[0] || null,
    oracleText: normalizeText(entry),
  }));
}

function mapBibliographyRowsToItems(entries, oracleRows, itemsById) {
  const textToIds = new Map();
  for (const row of oracleRows) {
    const existing = textToIds.get(row.oracleText) || [];
    existing.push(row.id);
    textToIds.set(row.oracleText, existing);
  }

  return entries.map((entry) => {
    let itemId = null;
    if (entry.oracle) {
      const matchingIds = textToIds.get(entry.oracle) || [];
      itemId = matchingIds.shift() || null;
    }

    return {
      ...entry,
      itemId,
      refType: itemId && itemsById[itemId] ? itemsById[itemId].type || null : null,
    };
  });
}

function summarizeIssueBuckets(entries) {
  const buckets = {};

  for (const entry of entries) {
    if (entry.match) continue;

    if (Array.isArray(entry.issues) && entry.issues.length > 0) {
      for (const issue of entry.issues) {
        const key = issue.component
          ? `${issue.component}:${issue.issue || 'unknown'}`
          : issue.issue || 'unknown';
        buckets[key] = (buckets[key] || 0) + 1;
      }
      continue;
    }

    buckets.unknown = (buckets.unknown || 0) + 1;
  }

  return buckets;
}

function summarizeOracleResult(result) {
  const bibliography = result?.bibliography || { passed: 0, total: 0, entries: [] };
  return {
    passed: bibliography.passed || 0,
    total: bibliography.total || 0,
    failed: bibliography.failed != null
      ? bibliography.failed
      : Math.max(0, (bibliography.total || 0) - (bibliography.passed || 0)),
    issueBuckets: summarizeIssueBuckets(bibliography.entries || []),
  };
}

function ensureDir(dirPath) {
  fs.mkdirSync(dirPath, { recursive: true });
}

function extractRichBenchmarkCluster(options, deps = {}) {
  const {
    runOracle = runOracleBibliography,
    renderOracleRows = renderOracleBibliographyRows,
    resolveStylePath = resolveLegacyStylePath,
  } = deps;

  const { styleData } = loadStyleData(options.style);
  const legacyStylePath = resolveStylePath(options.style, styleData);
  const benchmarkRun = resolveBenchmarkRun(options.style, options.benchmark);
  const fixtureItems = loadFixtureItems(benchmarkRun.refsFixture);
  const initiallySelectedItems = selectFixtureItems(fixtureItems, options);

  if (initiallySelectedItems.length === 0) {
    throw new Error('Selector matched zero fixture items');
  }

  const initialTemp = makeTempFixture(initiallySelectedItems);
  let initialOracleResult;
  let initialRows;
  try {
    initialOracleResult = runOracle(legacyStylePath, initialTemp.fixturePath);
    const itemsById = Object.fromEntries(initiallySelectedItems.map((item) => [item.id, item]));
    initialRows = mapBibliographyRowsToItems(
      initialOracleResult?.bibliography?.entries || [],
      renderOracleRows(legacyStylePath, initiallySelectedItems),
      itemsById
    );
  } finally {
    cleanupTempFixture(initialTemp);
  }

  const mismatchIds = new Set(
    initialRows
      .filter((entry) => !entry.match && entry.itemId)
      .map((entry) => entry.itemId)
  );
  const unresolvedMismatchCount = initialRows.filter((entry) => !entry.match && !entry.itemId).length;

  const finalItems = options.onlyMismatches
    ? (
      unresolvedMismatchCount > 0
        ? initiallySelectedItems
        : initiallySelectedItems.filter((item) => mismatchIds.has(item.id))
    )
    : initiallySelectedItems;

  const finalTemp = makeTempFixture(finalItems);
  let finalOracleResult;
  let finalRows;
  try {
    finalOracleResult = finalItems.length === 0
      ? { bibliography: { passed: 0, total: 0, failed: 0, entries: [] } }
      : runOracle(legacyStylePath, finalTemp.fixturePath);
    const itemsById = Object.fromEntries(finalItems.map((item) => [item.id, item]));
    finalRows = finalItems.length === 0
      ? []
      : mapBibliographyRowsToItems(
        finalOracleResult?.bibliography?.entries || [],
        renderOracleRows(legacyStylePath, finalItems),
        itemsById
      );
  } finally {
    cleanupTempFixture(finalTemp);
  }

  ensureDir(options.outDir);
  const clusterFixturePath = path.join(options.outDir, 'cluster-fixture.json');
  const clusterBeforePath = path.join(options.outDir, 'cluster-before.json');
  const clusterSummaryPath = path.join(options.outDir, 'cluster-summary.json');

  fs.writeFileSync(clusterFixturePath, JSON.stringify(createFixtureObject(finalItems), null, 2));

  const clusterBefore = {
    style: options.style,
    legacyStylePath: path.relative(PROJECT_ROOT, legacyStylePath),
    benchmark: {
      id: benchmarkRun.id,
      label: benchmarkRun.label,
      refsFixture: path.relative(PROJECT_ROOT, benchmarkRun.refsFixture),
      scope: benchmarkRun.scope,
      countTowardFidelity: benchmarkRun.countTowardFidelity,
    },
    selector: {
      types: options.types || null,
      ids: options.ids || null,
      onlyMismatches: options.onlyMismatches,
    },
    initialSelection: {
      count: initiallySelectedItems.length,
      ids: initiallySelectedItems.map((item) => item.id),
      types: [...new Set(initiallySelectedItems.map((item) => item.type).filter(Boolean))].sort(),
    },
    initialOracle: {
      bibliography: {
        passed: initialOracleResult?.bibliography?.passed || 0,
        total: initialOracleResult?.bibliography?.total || 0,
        failed: initialOracleResult?.bibliography?.failed || 0,
        entries: initialRows,
      },
    },
    mismatchIds: [...mismatchIds].sort(),
    unresolvedMismatchCount,
  };
  fs.writeFileSync(clusterBeforePath, JSON.stringify(clusterBefore, null, 2));

  const clusterSummary = {
    style: options.style,
    benchmark: benchmarkRun.id,
    selector: clusterBefore.selector,
    initialSelectionCount: initiallySelectedItems.length,
    clusterFixtureCount: finalItems.length,
    reductionApplied: options.onlyMismatches && unresolvedMismatchCount === 0,
    unresolvedMismatchCount,
    initialSummary: summarizeOracleResult({
      bibliography: {
        ...(initialOracleResult?.bibliography || {}),
        entries: initialRows,
      },
    }),
    clusterSummary: summarizeOracleResult({
      bibliography: {
        ...(finalOracleResult?.bibliography || {}),
        entries: finalRows,
      },
    }),
    clusterItemIds: finalItems.map((item) => item.id),
  };
  fs.writeFileSync(clusterSummaryPath, JSON.stringify(clusterSummary, null, 2));

  return {
    clusterFixturePath,
    clusterBeforePath,
    clusterSummaryPath,
    clusterSummary,
  };
}

function main() {
  const options = parseArgs();
  const result = extractRichBenchmarkCluster(options);
  console.log(JSON.stringify({
    fixture: path.relative(PROJECT_ROOT, result.clusterFixturePath),
    before: path.relative(PROJECT_ROOT, result.clusterBeforePath),
    summary: path.relative(PROJECT_ROOT, result.clusterSummaryPath),
    clusterSummary: result.clusterSummary,
  }, null, 2));
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = {
  extractRichBenchmarkCluster,
  loadFixtureItems,
  mapBibliographyRowsToItems,
  parseArgs,
  resolveBenchmarkRun,
  selectFixtureItems,
  summarizeIssueBuckets,
};
