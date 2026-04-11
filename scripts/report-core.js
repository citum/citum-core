#!/usr/bin/env node
/**
 * Core Styles Compatibility Report Generator
 *
 * Generates a JSON report of compatibility metrics for core styles in styles/
 * and optionally produces an HTML dashboard.
 *
 * Usage:
 *   node report-core.js                                      # Output JSON to stdout
 *   node report-core.js --write-html                         # Write HTML to docs/compat.html
 *   node report-core.js --output-html /path/to/output.html   # Write HTML to custom path
 *   node report-core.js --style chicago-author-date          # Output one official style report
 *   node report-core.js --styles-dir /path/to/csl            # Override CSL directory
 *   node report-core.js --styles chicago-author-date,apa-7th # Limit report to selected styles
 */

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const { execFileSync, spawn } = require('child_process');
const yaml = require('js-yaml');
const {
  loadFixtureSufficiency,
  loadVerificationPolicy,
  resolveScopeAuthority,
  resolveFixtureSufficiency,
  resolveVerificationPolicy,
  resolveStyleData,
} = require('./lib/verification-policy');
const {
  DEFAULT_CITATIONS_FIXTURE,
  DEFAULT_REFS_FIXTURE,
  FIXTURE_SET_CITATIONS,
  FIXTURE_SET_REFS,
  NOTE_CITATIONS_FIXTURE,
  getAdditionalFixtureSetNames,
  getEffectiveVerificationScopes,
  inferLegacySourceName,
} = require('./lib/style-verification');
const {
  getLineagePresentation,
  loadReportProvenance,
} = require('./lib/report-metadata');
const {
  auditNoteStyle,
  discoverNoteStyles,
} = require('./lib/note-position-audit');
const {
  compareText,
  normalizeText,
  textSimilarity,
} = require('./oracle-utils');
const { maybeDatasetErrorForFile } = require('./lib/dataset-guard');

const CUSTOM_TAG_SCHEMA = yaml.DEFAULT_SCHEMA.extend([
  new yaml.Type('!custom', {
    kind: 'mapping',
    construct(data) {
      return data || {};
    },
  }),
]);

const KNOWN_DEPENDENTS = {
  'apa-7th': 783,
  'elsevier-with-titles': 672,
  'elsevier-harvard': 665,
  'elsevier-vancouver': 502,
  'springer-vancouver-brackets': 472,
  'springer-basic-author-date': 460,
  'springer-basic-brackets': 352,
  'springer-socpsych-author-date': 317,
  'american-medical-association': 293,
  'taylor-and-francis-chicago-author-date': 234,
  'springer-mathphys-brackets': 201,
  'multidisciplinary-digital-publishing-institute': 180,
  'ieee': 176,
  'nlm-citation-sequence-superscript': 121,
  'nlm-citation-sequence': 116,
  'karger-journals': 85,
  'institute-of-physics-numeric': 82,
  'thieme-german': 74,
  'mary-ann-liebert-vancouver': 72,
  'biomed-central': 66,
  'chicago-notes': 5,
};

const SKIPPED_STYLES = ['alpha', 'iso690-author-date', 'iso690-numeric'];
const PROJECT_ROOT = path.dirname(__dirname);
const DEFAULT_REPORT_CACHE_DIR = path.join(PROJECT_ROOT, '.oracle-cache', 'report-core');
const DEFAULT_PARALLELISM = 4;
const DEFAULT_PROCESS_TIMEOUT_MS = 120000;
const CSL_SNAPSHOT_DIR = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'csl');
const BIBLATEX_SNAPSHOT_DIR = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'biblatex');
const COMPOUND_SNAPSHOT_DIR = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'compound');
const REPORT_CACHE_VERSION = 5;

const TOTAL_DEPENDENTS = 7987;
const CORE_FALLBACK_TYPES = [
  'article-journal',
  'book',
  'chapter',
  'report',
  'thesis',
  'paper-conference',
  'webpage',
];
const BENCHMARK_LABELS = {
  'citeproc-js': 'citeproc-js',
  'citeproc-js-live': 'citeproc-js',
  'citum-baseline': 'Citum baseline',
  biblatex: 'biblatex',
  documentary: 'documentary',
};

const BENCHMARK_RUNNERS = {
  CITEPROC_ORACLE: 'citeproc-oracle',
  NATIVE_SMOKE: 'native-smoke',
};

function consumeFlagValue(args, index, flag) {
  const value = args[index + 1];
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error(`Missing value for ${flag}`);
  }
  return {
    nextIndex: index + 1,
    value,
  };
}

/**
 * Parse command-line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    writeHtml: false,
    outputHtml: null,
    styleName: null,
    styleFile: null,
    stylesDir: null,
    styles: null,
    parallelism: DEFAULT_PARALLELISM,
    allowLiveFallback: false,
    timings: false,
    cacheDir: DEFAULT_REPORT_CACHE_DIR,
    citumBin: process.env.CITUM_BIN || null,
    caseSensitive: true,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--write-html') {
      options.writeHtml = true;
    } else if (args[i] === '--output-html') {
      const consumed = consumeFlagValue(args, i, '--output-html');
      i = consumed.nextIndex;
      options.outputHtml = consumed.value;
      options.writeHtml = true;
    } else if (args[i] === '--style') {
      const consumed = consumeFlagValue(args, i, '--style');
      i = consumed.nextIndex;
      options.styleName = consumed.value.trim();
    } else if (args[i] === '--style-file') {
      const consumed = consumeFlagValue(args, i, '--style-file');
      i = consumed.nextIndex;
      options.styleFile = path.resolve(consumed.value);
    } else if (args[i] === '--styles-dir') {
      const consumed = consumeFlagValue(args, i, '--styles-dir');
      i = consumed.nextIndex;
      options.stylesDir = consumed.value;
    } else if (args[i] === '--styles') {
      const consumed = consumeFlagValue(args, i, '--styles');
      i = consumed.nextIndex;
      options.styles = consumed.value
        .split(',')
        .map((style) => style.trim())
        .filter(Boolean);
      if (options.styles.length === 0) {
        throw new Error('Missing value for --styles');
      }
    } else if (args[i] === '--parallelism') {
      options.parallelism = Math.max(1, parseInt(args[++i], 10) || DEFAULT_PARALLELISM);
    } else if (args[i] === '--allow-live-fallback' || args[i] === '--refresh-missing') {
      options.allowLiveFallback = true;
    } else if (args[i] === '--timings') {
      options.timings = true;
    } else if (args[i] === '--cache-dir') {
      options.cacheDir = path.resolve(args[++i]);
    } else if (args[i] === '--citum-bin') {
      options.citumBin = path.resolve(args[++i]);
    } else if (args[i] === '--case-sensitive') {
      options.caseSensitive = true;
    } else if (args[i] === '--case-insensitive') {
      options.caseSensitive = false;
    }
  }

  if (options.styleName && options.styles?.length) {
    throw new Error('Flags --style and --styles are mutually exclusive');
  }
  if (options.styleFile && !options.styleName) {
    throw new Error('Flag --style-file requires --style');
  }

  return options;
}

/**
 * Get git short commit hash or 'unknown' on error
 */
function getGitCommit() {
  try {
    return execFileSync('git', ['rev-parse', '--short', 'HEAD'], {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
    }).trim();
  } catch {
    return 'unknown';
  }
}

/**
 * Get ISO timestamp
 */
function getTimestamp() {
  return new Date().toISOString();
}

/**
 * Find styles directory
 */
function getStylesDir(optionsDir) {
  if (optionsDir) return optionsDir;

  const defaultDir = path.join(PROJECT_ROOT, 'styles-legacy');

  if (fs.existsSync(defaultDir)) {
    return defaultDir;
  }

  throw new Error(`Styles directory not found. Use --styles-dir to specify path.`);
}

function ensureDir(dirPath) {
  fs.mkdirSync(dirPath, { recursive: true });
}

function hashContent(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function hashFile(filePath) {
  return hashContent(fs.readFileSync(filePath));
}

function equivalentText(expected, actual, options = {}) {
  return compareText(expected, actual, options).match;
}

function fixtureHash(refsFixture, citationsFixture) {
  const hash = crypto.createHash('sha256');
  hash.update(fs.readFileSync(refsFixture));
  hash.update(fs.readFileSync(citationsFixture));
  return hash.digest('hex').slice(0, 16);
}

function resolveCitumBinary(explicitPath = null) {
  const cargoTargetDir = process.env.CARGO_TARGET_DIR
    ? path.resolve(process.env.CARGO_TARGET_DIR)
    : null;
  const candidates = [
    explicitPath,
    process.env.CITUM_BIN,
    cargoTargetDir ? path.join(cargoTargetDir, 'debug', 'citum') : null,
    cargoTargetDir ? path.join(cargoTargetDir, 'release', 'citum') : null,
    path.join(PROJECT_ROOT, 'target', 'debug', 'citum'),
    path.join(PROJECT_ROOT, 'target', 'release', 'citum'),
  ].filter(Boolean);

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return path.resolve(candidate);
    }
  }

  execFileSync('cargo', ['build', '-q', '--bin', 'citum'], {
    cwd: PROJECT_ROOT,
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });

  const builtBinary = cargoTargetDir
    ? path.join(cargoTargetDir, 'debug', 'citum')
    : path.join(PROJECT_ROOT, 'target', 'debug', 'citum');
  if (!fs.existsSync(builtBinary)) {
    throw new Error(`Expected Citum binary after build: ${builtBinary}`);
  }
  return builtBinary;
}

function createTimerBucket() {
  return { count: 0, totalMs: 0, cacheHits: 0 };
}

function createReportRuntime(options = {}) {
  const cacheDir = path.resolve(options.cacheDir || DEFAULT_REPORT_CACHE_DIR);
  ensureDir(cacheDir);

  return {
    allowLiveFallback: Boolean(options.allowLiveFallback),
    cacheDir,
    caseSensitive: options.caseSensitive !== false,
    timings: new Map(),
    stylesDir: options.stylesDir ? path.resolve(options.stylesDir) : null,
    citumBin: resolveCitumBinary(options.citumBin),
    recordTiming(kind, durationMs, cacheHit = false) {
      const bucket = this.timings.get(kind) || createTimerBucket();
      bucket.count += 1;
      bucket.totalMs += durationMs;
      if (cacheHit) bucket.cacheHits += 1;
      this.timings.set(kind, bucket);
    },
  };
}

function spawnProcess(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd || PROJECT_ROOT,
      env: options.env || process.env,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let stderr = '';
    let settled = false;
    const timeoutMs = options.timeout ?? DEFAULT_PROCESS_TIMEOUT_MS;
    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      child.kill('SIGKILL');
      reject(new Error(`Command timed out: ${command} ${args.join(' ')}`));
    }, timeoutMs);

    child.stdout.on('data', (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });
    child.on('error', (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      reject(error);
    });
    child.on('close', (code) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      resolve({ code: code ?? 0, stdout, stderr });
    });
  });
}

async function runCachedJsonJob(runtime, config) {
  const keyJson = JSON.stringify({
    version: REPORT_CACHE_VERSION,
    ...config.cacheKey,
  });
  const cacheFile = path.join(runtime.cacheDir, `${hashContent(keyJson)}.json`);
  if (fs.existsSync(cacheFile)) {
    const started = Date.now();
    try {
      const cached = JSON.parse(fs.readFileSync(cacheFile, 'utf8'));
      runtime.recordTiming(config.kind, Date.now() - started, true);
      return cached;
    } catch {
      // Parallel report runs can observe partially written cache files.
      // Remove the corrupt cache entry and recompute it below.
      fs.rmSync(cacheFile, { force: true });
    }
  }

  const started = Date.now();
  const result = await config.compute();
  runtime.recordTiming(config.kind, Date.now() - started, false);
  const tempCacheFile = `${cacheFile}.${process.pid}.${Date.now()}.tmp`;
  fs.writeFileSync(tempCacheFile, JSON.stringify(result));
  fs.renameSync(tempCacheFile, cacheFile);
  return result;
}

async function mapWithConcurrency(items, limit, worker) {
  const results = new Array(items.length);
  let cursor = 0;

  async function runNext() {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= items.length) return;
      results[index] = await worker(items[index], index);
    }
  }

  const workers = Array.from({ length: Math.min(limit, items.length || 1) }, () => runNext());
  await Promise.all(workers);
  return results;
}

function serializeTimingSummary(runtime) {
  return Object.fromEntries(
    [...runtime.timings.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([kind, stats]) => [
        kind,
        {
          count: stats.count,
          cacheHits: stats.cacheHits,
          totalMs: parseFloat(stats.totalMs.toFixed(1)),
          averageMs: stats.count > 0 ? parseFloat((stats.totalMs / stats.count).toFixed(1)) : 0,
        },
      ])
  );
}

function inferStyleUpstream(styleData) {
  const sourceId = styleData?.info?.source?.['csl-id'];
  if (typeof sourceId !== 'string' || sourceId.trim().length === 0) {
    return 'unknown';
  }

  const normalized = sourceId.toLowerCase();
  if (normalized.includes('ctan.org/pkg/biblatex') || normalized.includes('biblatex')) {
    return 'biblatex';
  }
  if (normalized.includes('zotero.org/styles')) {
    return 'csl';
  }
  return 'unknown';
}

function inferLineageKey(styleData, hasLegacySource) {
  const upstream = inferStyleUpstream(styleData);
  const adaptedBy = styleData?.info?.source?.['adapted-by'];

  if (upstream === 'biblatex') return 'biblatex-derived';
  if (upstream === 'csl') return 'csl-derived';
  if (adaptedBy === 'citum-migrate') return 'csl-derived';
  if (hasLegacySource) return 'csl-derived';
  return 'citum-native';
}

function normalizeBenchmarkSource(authority) {
  if (!authority) return 'unknown';
  return authority === 'citeproc-js-live' ? 'citeproc-js' : authority;
}

function formatAuthorityLabel(authority, authorityId = null) {
  const normalized = normalizeBenchmarkSource(authority);
  if ((normalized === 'biblatex' || normalized === 'documentary') && authorityId) {
    return `${BENCHMARK_LABELS[normalized] || normalized}: ${authorityId}`;
  }
  return BENCHMARK_LABELS[normalized] || String(normalized);
}

function computeImpactPct(cslReach) {
  return cslReach != null
    ? (cslReach / TOTAL_DEPENDENTS * 100).toFixed(2)
    : null;
}

function discoverCoreStyles(provenanceConfig = loadReportProvenance()) {
  const stylesRoot = path.join(path.dirname(__dirname), 'styles');
  if (!fs.existsSync(stylesRoot)) {
    throw new Error(`Core styles directory not found: ${stylesRoot}`);
  }

  const styleFiles = fs.readdirSync(stylesRoot)
    .filter((entry) => entry.endsWith('.yaml'))
    .filter((entry) => !SKIPPED_STYLES.includes(path.basename(entry, '.yaml')))
    .sort((a, b) => a.localeCompare(b));

  if (styleFiles.length === 0) {
    throw new Error(`No style YAML files found in: ${stylesRoot}`);
  }

  return styleFiles.map((filename) => {
    const stylePath = path.join(stylesRoot, filename);
    const name = path.basename(filename, '.yaml');
    let rawStyleData = null;
    let styleData = null;

    try {
      rawStyleData = yaml.load(fs.readFileSync(stylePath, 'utf8'), { schema: CUSTOM_TAG_SCHEMA });
      styleData = resolveStyleData(rawStyleData);
    } catch {
      rawStyleData = null;
      styleData = null;
    }
    const sourceName = inferLegacySourceName(name, rawStyleData);

    const legacySourcePath = path.join(path.dirname(__dirname), 'styles-legacy', `${sourceName}.csl`);
    const hasLegacySource = fs.existsSync(legacySourcePath);
    const lineageKey = provenanceConfig.styles?.[name]?.lineage
      || inferLineageKey(styleData, hasLegacySource);
    const origin = getLineagePresentation(lineageKey, provenanceConfig);
    const cslReach = KNOWN_DEPENDENTS[name] ?? null;

    return {
      name,
      sourceName,
      cslReach,
      dependents: cslReach,
      format: inferStyleFormat(styleData),
      hasBibliography: hasBibliographyTemplate(styleData),
      originKey: origin.key,
      originLabel: origin.label,
      originSortRank: origin.sortRank,
    };
  });
}

function resolveSelectedStyles(coreStyles, selectedNames = null) {
  if (!Array.isArray(selectedNames) || selectedNames.length === 0) {
    return coreStyles;
  }

  const requested = new Set(selectedNames);
  const known = new Set(coreStyles.map((style) => style.name));
  const unknown = [...requested].filter((name) => !known.has(name)).sort((a, b) => a.localeCompare(b));
  if (unknown.length > 0) {
    throw new Error(`Unknown style name(s) for --styles: ${unknown.join(', ')}`);
  }

  return coreStyles.filter((style) => requested.has(style.name));
}

function buildNoteStyleLookup() {
  return new Map(discoverNoteStyles().map((style) => [style.name, style]));
}

function selectPrimaryComparator(styleSpec, verificationPolicy) {
  return verificationPolicy.authority;
}

function inferStyleFormat(styleData) {
  const processing = styleData?.options?.processing;
  if (typeof processing === 'string') {
    return processing;
  }
  if (processing && typeof processing === 'object') {
    if (Object.prototype.hasOwnProperty.call(processing, 'note')) {
      return 'note';
    }
    if (Object.prototype.hasOwnProperty.call(processing, 'author-date')) {
      return 'author-date';
    }
    if (
      Object.prototype.hasOwnProperty.call(processing, 'label') ||
      Object.prototype.hasOwnProperty.call(processing, 'numeric')
    ) {
      return 'numeric';
    }
    // !custom processing objects with sort/group/disambiguate keys are author-date
    if (
      Object.prototype.hasOwnProperty.call(processing, 'sort') ||
      Object.prototype.hasOwnProperty.call(processing, 'group') ||
      Object.prototype.hasOwnProperty.call(processing, 'disambiguate')
    ) {
      return 'author-date';
    }
  }

  const citation = styleData?.citation || {};
  const candidateTemplates = [
    citation.template,
    citation.integral?.template,
    citation['non-integral']?.template,
  ].filter(Array.isArray);
  const usesCitationNumbers = candidateTemplates.some((template) =>
    flattenTemplateComponents(template).some((component) => Boolean(component?.number))
  );

  if (usesCitationNumbers) return 'numeric';

  // Styles with non-integral/integral citation sections are always author-date
  if (citation['non-integral'] || citation['integral']) return 'author-date';

  // Styles whose citation template uses contributor + date (but no number) are author-date
  const allTemplates = candidateTemplates.concat(
    [citation.template].filter(Array.isArray)
  );
  const flat = allTemplates.flatMap((t) => flattenTemplateComponents(t));
  const hasContributor = flat.some((c) => Boolean(c?.contributor));
  const hasDate = flat.some((c) => Boolean(c?.date));
  if (hasContributor && hasDate) return 'author-date';

  return 'unknown';
}

function hasBibliographyTemplate(styleData) {
  const bibliography = styleData?.bibliography;
  if (!bibliography || typeof bibliography !== 'object') {
    return false;
  }

  const hasTemplate = Array.isArray(bibliography.template) && bibliography.template.length > 0;
  const typeTemplates = bibliography['type-templates'];
  const hasTypeTemplates = Boolean(typeTemplates && Object.keys(typeTemplates).length > 0);
  return hasTemplate || hasTypeTemplates;
}

function buildEmptyOracleResult(overrides = {}) {
  return {
    citations: { passed: 0, total: 0, entries: [] },
    bibliography: { passed: 0, total: 0, entries: [] },
    adjusted: {
      citations: { passed: 0, total: 0, entries: [] },
      bibliography: { passed: 0, total: 0, entries: [] },
      divergenceSummary: {},
    },
    citationsByType: {},
    componentSummary: {},
    ...overrides,
  };
}

function cloneOracleSection(section = {}) {
  return {
    passed: section.passed || 0,
    total: section.total || 0,
    entries: [...(section.entries || [])],
  };
}

function cloneOracleResult(oracleResult = {}) {
  return {
    citations: cloneOracleSection(oracleResult.citations),
    bibliography: cloneOracleSection(oracleResult.bibliography),
    adjusted: {
      citations: cloneOracleSection(oracleResult.adjusted?.citations),
      bibliography: cloneOracleSection(oracleResult.adjusted?.bibliography),
      divergenceSummary: { ...(oracleResult.adjusted?.divergenceSummary || {}) },
    },
    citationsByType: { ...(oracleResult.citationsByType || {}) },
    componentSummary: { ...(oracleResult.componentSummary || {}) },
    oracleSource: oracleResult.oracleSource || null,
    error: oracleResult.error || null,
  };
}

function getEffectiveOracleSection(oracleResult, sectionName) {
  const adjustedSection = oracleResult?.adjusted?.[sectionName];
  if (adjustedSection) {
    const adjustedTotal = Number(adjustedSection.total || 0);
    const adjustedPassed = Number(adjustedSection.passed || 0);
    const hasDivergences = Object.keys(oracleResult?.adjusted?.divergenceSummary || {}).length > 0;
    if (adjustedTotal > 0 || adjustedPassed > 0 || hasDivergences) {
      return adjustedSection;
    }
  }
  return oracleResult?.[sectionName] || { passed: 0, total: 0, entries: [] };
}

function countCaseMismatches(section) {
  return (section?.entries || []).filter((entry) => entry?.caseMismatch === true).length;
}

function collectCaseMismatchSummary(oracleResult) {
  const citations = getEffectiveOracleSection(oracleResult, 'citations');
  const bibliography = getEffectiveOracleSection(oracleResult, 'bibliography');
  const citationCount = countCaseMismatches(citations);
  const bibliographyCount = countCaseMismatches(bibliography);
  return {
    citations: citationCount,
    bibliography: bibliographyCount,
    total: citationCount + bibliographyCount,
  };
}

function mergeDivergenceDetails(base = {}, extra = {}) {
  const merged = { ...base };

  for (const [key, value] of Object.entries(extra)) {
    if (Array.isArray(value)) {
      const current = Array.isArray(merged[key]) ? merged[key] : [];
      merged[key] = [...new Set([...current, ...value])];
      continue;
    }

    if (typeof value === 'number') {
      merged[key] = (typeof merged[key] === 'number' ? merged[key] : 0) + value;
      continue;
    }

    if (typeof value === 'boolean') {
      merged[key] = Boolean(merged[key]) || value;
      continue;
    }

    merged[key] = value;
  }

  return merged;
}

function mergeDivergenceSummaries(...summaries) {
  const merged = {};

  for (const summary of summaries) {
    for (const [divergenceId, details] of Object.entries(summary || {})) {
      merged[divergenceId] = mergeDivergenceDetails(merged[divergenceId], details);
    }
  }

  return merged;
}

function getCslSnapshotStatus(stylePath, refsFixture, citationsFixture) {
  const styleName = path.basename(stylePath, '.csl');
  const snapshotPath = path.join(CSL_SNAPSHOT_DIR, `${styleName}.json`);
  if (!fs.existsSync(snapshotPath)) {
    return {
      ok: false,
      status: 'missing',
      message: `Snapshot missing for ${styleName}. Run: node scripts/oracle-snapshot.js ${stylePath}`,
      snapshotPath,
    };
  }

  let snapshot;
  try {
    snapshot = JSON.parse(fs.readFileSync(snapshotPath, 'utf8'));
  } catch (error) {
    return {
      ok: false,
      status: 'invalid',
      message: `Snapshot parse failed for ${styleName}: ${error.message}`,
      snapshotPath,
    };
  }

  const expectedHash = fixtureHash(refsFixture, citationsFixture);
  if (snapshot.fixture_hash !== expectedHash) {
    return {
      ok: false,
      status: 'stale',
      message: `Snapshot stale for ${styleName}. Run: node scripts/oracle-snapshot.js ${stylePath}`,
      snapshotPath,
    };
  }

  return { ok: true, status: 'ok', snapshotPath };
}

async function runNodeOracleScript(scriptPath, args) {
  const result = await spawnProcess(process.execPath, [scriptPath, ...args]);
  return result;
}

function resolveBenchmarkFixturePath(filePath) {
  return path.isAbsolute(filePath) ? filePath : path.join(PROJECT_ROOT, filePath);
}

function resolveBenchmarkRunConfig(benchmarkRun) {
  return {
    ...benchmarkRun,
    refsFixture: resolveBenchmarkFixturePath(benchmarkRun.refsFixture),
    citationsFixture: benchmarkRun.citationsFixture
      ? resolveBenchmarkFixturePath(benchmarkRun.citationsFixture)
      : null,
  };
}

function toRepoRelativePath(filePath) {
  if (!filePath) return null;
  const relativePath = path.relative(PROJECT_ROOT, filePath);
  return relativePath && !relativePath.startsWith('..')
    ? relativePath
    : filePath;
}

async function runCiteprocSnapshotOracle(runtime, stylePath, styleName, styleFormat, refsFixture = DEFAULT_REFS_FIXTURE, citationsFixture = null) {
  const resolvedCitationsFixture = citationsFixture
    || (styleFormat === 'note' ? NOTE_CITATIONS_FIXTURE : DEFAULT_CITATIONS_FIXTURE);
  const snapshotStatus = getCslSnapshotStatus(stylePath, refsFixture, resolvedCitationsFixture);
  const fastScript = path.join(__dirname, 'oracle-fast.js');
  const liveScript = path.join(__dirname, 'oracle.js');
  const cacheKey = {
    backend: 'citeprocSnapshot',
    styleName,
    stylePath,
    refsFixture,
    citationsFixture: resolvedCitationsFixture,
    styleHash: hashFile(stylePath),
    refsHash: hashFile(refsFixture),
    citationsHash: hashFile(resolvedCitationsFixture),
    snapshotStatus: snapshotStatus.status,
    snapshotHash: snapshotStatus.ok ? hashFile(snapshotStatus.snapshotPath) : null,
    fastScriptHash: hashFile(fastScript),
    liveScriptHash: hashFile(liveScript),
    citumBin: runtime.citumBin,
    citumBinHash: hashFile(runtime.citumBin),
    allowLiveFallback: runtime.allowLiveFallback,
    caseSensitive: runtime.caseSensitive,
  };

  return runCachedJsonJob(runtime, {
    kind: 'citeprocSnapshot',
    cacheKey,
    async compute() {
      const fastArgs = [
        stylePath,
        '--json',
        '--refs-fixture',
        refsFixture,
        '--citations-fixture',
        resolvedCitationsFixture,
        runtime.caseSensitive ? '--case-sensitive' : '--case-insensitive',
      ];

      if (snapshotStatus.ok) {
        const fast = await runNodeOracleScript(fastScript, fastArgs);
        if ((fast.code === 0 || fast.code === 1) && fast.stdout.trim()) {
          try {
            return JSON.parse(fast.stdout);
          } catch (error) {
            return buildEmptyOracleResult({
              error: `Snapshot oracle JSON parse failed for ${styleName}: ${error.message}`,
              style: styleName,
              oracleSource: 'citeproc-js',
            });
          }
        }

        const fastFailure = (fast.stderr || fast.stdout || `exit ${fast.code}`).trim();
        if (!runtime.allowLiveFallback) {
          return buildEmptyOracleResult({
            error: `Snapshot oracle failed for ${styleName}: ${fastFailure}`,
            style: styleName,
            oracleSource: 'citeproc-js',
            snapshotStatus: snapshotStatus.status,
          });
        }
      }

      if (!runtime.allowLiveFallback) {
        return buildEmptyOracleResult({
          error: snapshotStatus.message || `Snapshot oracle unavailable for ${styleName}`,
          style: styleName,
          oracleSource: 'citeproc-js',
          snapshotStatus: snapshotStatus.status,
        });
      }

      const live = await runNodeOracleScript(liveScript, fastArgs);
      if ((live.code === 0 || live.code === 1) && live.stdout.trim()) {
        try {
          const parsed = JSON.parse(live.stdout);
          parsed.oracleSource = 'citeproc-js-live';
          return parsed;
        } catch (error) {
          return buildEmptyOracleResult({
            error: `Live oracle JSON parse failed for ${styleName}: ${error.message}`,
            style: styleName,
            oracleSource: 'citeproc-js-live',
          });
        }
      }

      return buildEmptyOracleResult({
        error: `Live oracle failed for ${styleName}: ${live.stderr || live.stdout || `exit ${live.code}`}`.trim(),
        style: styleName,
        oracleSource: 'citeproc-js-live',
      });
    },
  });
}

async function runNativeOracle(runtime, styleName) {
  const scriptPath = path.join(__dirname, 'oracle-native.js');
  const styleYamlPath = path.join(PROJECT_ROOT, 'styles', `${styleName}.yaml`);
  const fixturePath = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'compound-numeric-refs.yaml');
  const snapshotPath = path.join(COMPOUND_SNAPSHOT_DIR, `${styleName}.txt`);

  return runCachedJsonJob(runtime, {
    kind: 'nativeSnapshot',
    cacheKey: {
      backend: 'nativeSnapshot',
      styleName,
      styleYamlPath,
      fixturePath,
      snapshotPath,
      styleHash: hashFile(styleYamlPath),
      fixtureHash: hashFile(fixturePath),
      snapshotHash: fs.existsSync(snapshotPath) ? hashFile(snapshotPath) : null,
      scriptHash: hashFile(scriptPath),
      citumBin: runtime.citumBin,
      citumBinHash: hashFile(runtime.citumBin),
    },
    async compute() {
      const result = await runNodeOracleScript(scriptPath, [
        styleName,
        styleYamlPath,
        fixturePath,
        COMPOUND_SNAPSHOT_DIR,
      ]);
      if ((result.code === 0 || result.code === 1) && result.stdout.trim()) {
        try {
          return JSON.parse(result.stdout);
        } catch (error) {
          return buildEmptyOracleResult({
            error: `Native oracle parse failed: ${error.message}`,
            style: styleName,
            oracleSource: 'citum-baseline',
          });
        }
      }
      return buildEmptyOracleResult({
        error: `Native oracle failed: ${result.stderr || result.stdout || `exit ${result.code}`}`.trim(),
        style: styleName,
        oracleSource: 'citum-baseline',
      });
    },
  });
}

async function runCiteprocBenchmarkOracle(runtime, stylePath, styleName, benchmarkRun) {
  const resolvedRun = resolveBenchmarkRunConfig(benchmarkRun);
  const liveScript = path.join(__dirname, 'oracle.js');
  return runCachedJsonJob(runtime, {
    kind: 'benchmarkCiteproc',
    cacheKey: {
      backend: 'benchmarkCiteproc',
      styleName,
      stylePath,
      benchmarkRunId: resolvedRun.id,
      benchmarkRunner: resolvedRun.runner,
      scope: resolvedRun.scope,
      refsFixture: resolvedRun.refsFixture,
      citationsFixture: resolvedRun.citationsFixture,
      styleHash: hashFile(stylePath),
      refsHash: hashFile(resolvedRun.refsFixture),
      citationsHash: resolvedRun.citationsFixture ? hashFile(resolvedRun.citationsFixture) : null,
      liveScriptHash: hashFile(liveScript),
      citumBin: runtime.citumBin,
      citumBinHash: hashFile(runtime.citumBin),
      caseSensitive: runtime.caseSensitive,
    },
    async compute() {
      const args = [
        stylePath,
        '--json',
        '--scope', resolvedRun.scope,
        '--refs-fixture', resolvedRun.refsFixture,
        runtime.caseSensitive ? '--case-sensitive' : '--case-insensitive',
      ];
      if (resolvedRun.citationsFixture && resolvedRun.scope !== 'bibliography') {
        args.push('--citations-fixture', resolvedRun.citationsFixture);
      }
      const result = await runNodeOracleScript(liveScript, args);
      if ((result.code === 0 || result.code === 1) && result.stdout.trim()) {
        try {
          const parsed = JSON.parse(result.stdout);
          parsed.oracleSource = 'citeproc-js-live';
          return parsed;
        } catch (error) {
          return buildEmptyOracleResult({
            error: `Benchmark oracle JSON parse failed for ${styleName}/${resolvedRun.id}: ${error.message}`,
            style: styleName,
            oracleSource: 'citeproc-js-live',
          });
        }
      }

      return buildEmptyOracleResult({
        error: `Benchmark oracle failed for ${styleName}/${resolvedRun.id}: ${result.stderr || result.stdout || `exit ${result.code}`}`.trim(),
        style: styleName,
        oracleSource: 'citeproc-js-live',
      });
    },
  });
}

function countRenderedBibliographyEntries(rendered) {
  return rendered?.bibliography?.entries?.length || 0;
}

async function runNativeSmokeBenchmark(runtime, styleYamlPath, benchmarkRun) {
  const resolvedRun = resolveBenchmarkRunConfig(benchmarkRun);
  const rendered = await renderCitumJson(runtime, styleYamlPath, resolvedRun.refsFixture, 'bib');
  return {
    bibliographyEntries: countRenderedBibliographyEntries(rendered),
  };
}

async function renderCitumJson(runtime, styleYamlPath, refsFixture, mode = 'both', citationsFixture = null) {
  const resolvedCitationsFixture = citationsFixture ? path.resolve(citationsFixture) : null;
  const cacheKey = {
    backend: 'citumRender',
    styleYamlPath,
    refsFixture,
    citationsFixture: resolvedCitationsFixture,
    mode,
    styleHash: hashFile(styleYamlPath),
    refsHash: hashFile(refsFixture),
    citationsHash: resolvedCitationsFixture ? hashFile(resolvedCitationsFixture) : null,
    citumBin: runtime.citumBin,
    citumBinHash: hashFile(runtime.citumBin),
  };

  return runCachedJsonJob(runtime, {
    kind: 'citumRender',
    cacheKey,
    async compute() {
      const args = [
        'render', 'refs',
        '-b', refsFixture,
        '-s', styleYamlPath,
        '--json',
        '--mode', mode,
      ];
      if (resolvedCitationsFixture) {
        args.splice(4, 0, '-c', resolvedCitationsFixture);
      }
      const result = await spawnProcess(runtime.citumBin, args, { cwd: PROJECT_ROOT });
      if (result.code !== 0) {
        throw new Error((result.stderr || result.stdout || `exit ${result.code}`).trim());
      }
      return JSON.parse(result.stdout);
    },
  });
}

/**
 * Expand compound bibliography blocks into individual entries.
 *
 * Some biblatex styles (chem-acs, chem-rsc, chem-biochem) produce compound
 * bibliography entries where multiple sub-entries are merged into a single
 * string: "(1) text1 (2) text2 ... (N) textN".  Citum renders individual
 * entries, so we expand the compound blocks before comparing.
 */
function expandCompoundBibEntries(entries) {
  const result = [];
  for (const entry of entries) {
    // Compound block: starts with "(N) " and contains at least one more " (M) "
    if (/^\(\d+\) /.test(entry) && / \(\d+\) /.test(entry)) {
      // Split at positions where a space is immediately followed by "(M) "
      const subEntries = entry.split(/(?= \(\d+\) )/).map((s) => s.trim()).filter(Boolean);
      if (subEntries.length > 1) {
        result.push(...subEntries);
        continue;
      }
    }
    result.push(entry);
  }
  return result;
}

async function runBiblatexSnapshotOracle(runtime, styleName, styleYamlPath, authorityId) {
  const snapshotPath = path.join(BIBLATEX_SNAPSHOT_DIR, `${styleName}.json`);

  if (!fs.existsSync(snapshotPath)) {
    return buildEmptyOracleResult({
      error: `Biblatex snapshot not found: ${snapshotPath}`,
      oracleSource: 'biblatex',
      authorityId,
    });
  }

  return runCachedJsonJob(runtime, {
    kind: 'biblatexSnapshot',
    cacheKey: {
      backend: 'biblatexSnapshot',
      styleName,
      authorityId,
      styleYamlPath,
      refsFixture: DEFAULT_REFS_FIXTURE,
      snapshotPath,
      styleHash: hashFile(styleYamlPath),
      refsHash: hashFile(DEFAULT_REFS_FIXTURE),
      snapshotHash: hashFile(snapshotPath),
      caseSensitive: runtime.caseSensitive,
    },
    async compute() {
      let snapshot;
      try {
        snapshot = JSON.parse(fs.readFileSync(snapshotPath, 'utf8'));
      } catch (error) {
        return buildEmptyOracleResult({
          error: `Biblatex snapshot parse failed: ${error.message}`,
          oracleSource: 'biblatex',
          authorityId,
        });
      }

      try {
        // Build ordered type list from the fixture so we can track per-type pass rates.
        // The biblatex snapshot entries correspond positionally to fixture entries.
        const fixtureData = JSON.parse(fs.readFileSync(DEFAULT_REFS_FIXTURE, 'utf8'));
        const fixtureTypes = Object.entries(fixtureData)
          .filter(([key]) => key !== 'comment')
          .map(([, ref]) => (typeof ref === 'object' && ref !== null ? ref.type : null) || 'unknown');

        const rendered = await renderCitumJson(runtime, styleYamlPath, DEFAULT_REFS_FIXTURE, 'bib');
        const actualEntries = rendered?.bibliography?.entries?.map((entry) => entry.text) || [];
        const expectedEntries = expandCompoundBibEntries(snapshot.bibliography || []);
        const total = Math.max(expectedEntries.length, actualEntries.length);
        let passed = 0;
        const entries = [];
        const citationsByType = {};

        for (let i = 0; i < total; i++) {
          const expected = expectedEntries[i] ?? '';
          const actual = actualEntries[i] ?? '';
          const comparison = compareText(expected, actual, {
            caseSensitive: runtime.caseSensitive,
          });
          const match = comparison.match;
          if (match) passed += 1;
          entries.push({
            expected: comparison.expected,
            actual: comparison.actual,
            match,
            caseMismatch: comparison.caseMismatch,
          });

          // Track per-type stats using the fixture entry at this position.
          const refType = fixtureTypes[i] || 'unknown';
          const typeStats = citationsByType[refType] || { passed: 0, total: 0 };
          typeStats.total += 1;
          if (match) typeStats.passed += 1;
          citationsByType[refType] = typeStats;
        }

        return buildEmptyOracleResult({
          citations: { passed: 0, total: 0, entries: [] },
          bibliography: { passed, total, entries },
          citationsByType,
          oracleSource: 'biblatex',
          authorityId,
        });
      } catch (error) {
        return buildEmptyOracleResult({
          error: `Biblatex comparator failed: ${error.message}`,
          oracleSource: 'biblatex',
          authorityId,
        });
      }
    },
  });
}

async function runFamilyFixtureOracle(runtime, stylePath, styleName, fixtureSetName) {
  const refsFixture = FIXTURE_SET_REFS[fixtureSetName];
  const citationsFixture = FIXTURE_SET_CITATIONS[fixtureSetName];
  if (!refsFixture || !citationsFixture) return null;
  if (!fs.existsSync(refsFixture) || !fs.existsSync(citationsFixture)) return null;

  const liveScript = path.join(__dirname, 'oracle.js');
  return runCachedJsonJob(runtime, {
    kind: 'familyFixture',
    cacheKey: {
      backend: 'familyFixture',
      styleName,
      stylePath,
      refsFixture,
      citationsFixture,
      styleHash: hashFile(stylePath),
      refsHash: hashFile(refsFixture),
      citationsHash: hashFile(citationsFixture),
      liveScriptHash: hashFile(liveScript),
      citumBin: runtime.citumBin,
      citumBinHash: hashFile(runtime.citumBin),
      caseSensitive: runtime.caseSensitive,
    },
    async compute() {
      const result = await runNodeOracleScript(liveScript, [
        stylePath,
        '--json',
        '--refs-fixture', refsFixture,
        '--citations-fixture', citationsFixture,
        runtime.caseSensitive ? '--case-sensitive' : '--case-insensitive',
      ]);
      if ((result.code === 0 || result.code === 1) && result.stdout.trim()) {
        try {
          return JSON.parse(result.stdout);
        } catch (error) {
          process.stderr.write(`[family-fixture] ${styleName}/${fixtureSetName}: ${error.message}\n`);
          return null;
        }
      }
      process.stderr.write(`[family-fixture] ${styleName}/${fixtureSetName}: ${result.stderr || result.stdout || `exit ${result.code}`}\n`);
      return null;
    },
  });
}

/**
 * Merge a family fixture oracle result into the main oracle result.
 * Adds passed/total counts from the family run to the main result.
 */
function mergeOracleResults(main, extra) {
  if (!extra) return main;

  const mCit = main.citations || { passed: 0, total: 0 };
  const eCit = extra.citations || { passed: 0, total: 0 };
  const mBib = main.bibliography || { passed: 0, total: 0 };
  const eBib = extra.bibliography || { passed: 0, total: 0 };
  const mAdjCit = getEffectiveOracleSection(main, 'citations');
  const eAdjCit = getEffectiveOracleSection(extra, 'citations');
  const mAdjBib = getEffectiveOracleSection(main, 'bibliography');
  const eAdjBib = getEffectiveOracleSection(extra, 'bibliography');

  main.citations = {
    passed: (mCit.passed || 0) + (eCit.passed || 0),
    total: (mCit.total || 0) + (eCit.total || 0),
    entries: [...(mCit.entries || []), ...(eCit.entries || [])],
  };
  main.bibliography = {
    passed: (mBib.passed || 0) + (eBib.passed || 0),
    total: (mBib.total || 0) + (eBib.total || 0),
    entries: [...(mBib.entries || []), ...(eBib.entries || [])],
  };
  main.adjusted = {
    citations: {
      passed: (mAdjCit.passed || 0) + (eAdjCit.passed || 0),
      total: (mAdjCit.total || 0) + (eAdjCit.total || 0),
      entries: [...(mAdjCit.entries || []), ...(eAdjCit.entries || [])],
    },
    bibliography: {
      passed: (mAdjBib.passed || 0) + (eAdjBib.passed || 0),
      total: (mAdjBib.total || 0) + (eAdjBib.total || 0),
      entries: [...(mAdjBib.entries || []), ...(eAdjBib.entries || [])],
    },
    divergenceSummary: mergeDivergenceSummaries(
      main.adjusted?.divergenceSummary,
      extra.adjusted?.divergenceSummary
    ),
  };

  return main;
}

function mergeCitationResults(main, extra) {
  if (!extra) return main;

  const mainCitations = main.citations || { passed: 0, total: 0, entries: [] };
  const extraCitations = extra.citations || { passed: 0, total: 0, entries: [] };
  const mainAdjusted = getEffectiveOracleSection(main, 'citations');
  const extraAdjusted = getEffectiveOracleSection(extra, 'citations');
  main.citations = {
    passed: (mainCitations.passed || 0) + (extraCitations.passed || 0),
    total: (mainCitations.total || 0) + (extraCitations.total || 0),
    entries: [...(mainCitations.entries || []), ...(extraCitations.entries || [])],
  };
  main.adjusted = {
    citations: {
      passed: (mainAdjusted.passed || 0) + (extraAdjusted.passed || 0),
      total: (mainAdjusted.total || 0) + (extraAdjusted.total || 0),
      entries: [...(mainAdjusted.entries || []), ...(extraAdjusted.entries || [])],
    },
    bibliography: getEffectiveOracleSection(main, 'bibliography'),
    divergenceSummary: mergeDivergenceSummaries(
      main.adjusted?.divergenceSummary,
      extra.adjusted?.divergenceSummary
    ),
  };

  const mergedByType = { ...(main.citationsByType || {}) };
  for (const [refType, stats] of Object.entries(extra.citationsByType || {})) {
    const current = mergedByType[refType] || { passed: 0, total: 0 };
    mergedByType[refType] = {
      passed: (current.passed || 0) + (stats.passed || 0),
      total: (current.total || 0) + (stats.total || 0),
    };
  }
  main.citationsByType = mergedByType;

  return main;
}

function mergeOracleErrors(...results) {
  return results
    .map((result) => result?.error)
    .filter(Boolean)
    .join('\n');
}

async function executeBenchmarkRuns(benchmarkRuns, executor) {
  const results = [];
  for (const benchmarkRun of benchmarkRuns || []) {
    results.push(await executor(benchmarkRun));
  }
  return results;
}

function mergeBenchmarkRunIntoOracle(oracleResult, benchmarkRunRecord) {
  if (!benchmarkRunRecord?.countTowardFidelity || !benchmarkRunRecord?.oracleResult) {
    return oracleResult;
  }
  if (benchmarkRunRecord.scope === 'citation') {
    mergeCitationResults(oracleResult, benchmarkRunRecord.oracleResult);
    return oracleResult;
  }
  mergeOracleResults(oracleResult, benchmarkRunRecord.oracleResult);
  return oracleResult;
}

function formatBenchmarkRunRecord(benchmarkRun, extras = {}) {
  return {
    id: benchmarkRun.id,
    label: benchmarkRun.label,
    runner: benchmarkRun.runner,
    scope: benchmarkRun.scope,
    countTowardFidelity: benchmarkRun.countTowardFidelity,
    minPassRate: benchmarkRun.minPassRate ?? null,
    refsFixture: benchmarkRun.refsFixture,
    citationsFixture: benchmarkRun.citationsFixture || null,
    ...extras,
  };
}

function summarizeSection(section = null) {
  if (!section) return null;
  const total = section.total || 0;
  const passed = section.passed || 0;
  return {
    passed,
    total,
    failed: Math.max(0, total - passed),
    matchRate: total > 0 ? parseFloat((passed / total).toFixed(3)) : null,
  };
}

function summarizeBenchmarkRunRecord(benchmarkRunRecord) {
  return {
    id: benchmarkRunRecord.id,
    label: benchmarkRunRecord.label,
    runner: benchmarkRunRecord.runner,
    scope: benchmarkRunRecord.scope,
    status: benchmarkRunRecord.status,
    countTowardFidelity: benchmarkRunRecord.countTowardFidelity,
    minPassRate: benchmarkRunRecord.minPassRate ?? null,
    citations: summarizeSection(benchmarkRunRecord.citations),
    bibliography: summarizeSection(benchmarkRunRecord.bibliography),
    bibliographyEntries: benchmarkRunRecord.bibliographyEntries ?? null,
  };
}

function buildRichInputEvidenceSummary(styleRecord, benchmarkRunResults) {
  return {
    headlineGate: {
      citations: summarizeSection(styleRecord.rawCitations),
      bibliography: summarizeSection(styleRecord.rawBibliography),
      qualityScore: styleRecord.qualityScore,
    },
    officialSupplemental: benchmarkRunResults.map(summarizeBenchmarkRunRecord),
  };
}

function toPublishedBenchmarkRunRecord(benchmarkRunRecord) {
  return {
    id: benchmarkRunRecord.id,
    label: benchmarkRunRecord.label,
    runner: benchmarkRunRecord.runner,
    scope: benchmarkRunRecord.scope,
    countTowardFidelity: benchmarkRunRecord.countTowardFidelity,
    minPassRate: benchmarkRunRecord.minPassRate ?? null,
    refsFixture: toRepoRelativePath(benchmarkRunRecord.refsFixture),
    citationsFixture: toRepoRelativePath(benchmarkRunRecord.citationsFixture),
    status: benchmarkRunRecord.status,
    error: benchmarkRunRecord.error || null,
    citations: benchmarkRunRecord.citations
      ? {
        passed: benchmarkRunRecord.citations.passed || 0,
        total: benchmarkRunRecord.citations.total || 0,
      }
      : null,
    bibliography: benchmarkRunRecord.bibliography
      ? {
        passed: benchmarkRunRecord.bibliography.passed || 0,
        total: benchmarkRunRecord.bibliography.total || 0,
      }
      : null,
    bibliographyEntries: benchmarkRunRecord.bibliographyEntries ?? null,
  };
}

async function runBenchmarkRun(runtime, styleSpec, stylePath, styleYamlPath, benchmarkRun) {
  const resolvedRun = resolveBenchmarkRunConfig(benchmarkRun);
  try {
    if (resolvedRun.runner === BENCHMARK_RUNNERS.CITEPROC_ORACLE) {
      const oracleResult = await runCiteprocBenchmarkOracle(runtime, stylePath, styleSpec.name, resolvedRun);
      let benchmarkStatus;
      if (oracleResult.error) {
        benchmarkStatus = 'error';
      } else if (resolvedRun.minPassRate != null) {
        const bib = oracleResult.bibliography || { passed: 0, total: 0 };
        const cit = oracleResult.citations || { passed: 0, total: 0 };
        const totalPassed = (bib.passed || 0) + (cit.passed || 0);
        const totalItems = (bib.total || 0) + (cit.total || 0);
        const matchRate = totalItems > 0 ? totalPassed / totalItems : 0;
        benchmarkStatus = matchRate >= resolvedRun.minPassRate ? 'pass' : 'fail';
      } else {
        benchmarkStatus = 'ok';
      }
      return formatBenchmarkRunRecord(resolvedRun, {
        status: benchmarkStatus,
        error: oracleResult.error || null,
        oracleResult,
        citations: oracleResult.citations || { passed: 0, total: 0, entries: [] },
        bibliography: oracleResult.bibliography || { passed: 0, total: 0, entries: [] },
      });
    }

    if (resolvedRun.runner === BENCHMARK_RUNNERS.NATIVE_SMOKE) {
      const nativeResult = await runNativeSmokeBenchmark(runtime, styleYamlPath, resolvedRun);
      return formatBenchmarkRunRecord(resolvedRun, {
        status: 'pass',
        error: null,
        bibliographyEntries: nativeResult.bibliographyEntries,
      });
    }

    return formatBenchmarkRunRecord(resolvedRun, {
      status: 'error',
      error: `Unsupported benchmark runner: ${resolvedRun.runner}`,
    });
  } catch (error) {
    return formatBenchmarkRunRecord(resolvedRun, {
      status: 'error',
      error: error.message,
    });
  }
}

function composeScopedOracleResult(citationResult, bibliographyResult) {
  const combinedError = mergeOracleErrors(citationResult, bibliographyResult);
  return {
    citations: citationResult?.citations || { passed: 0, total: 0, entries: [] },
    bibliography: bibliographyResult?.bibliography || { passed: 0, total: 0, entries: [] },
    adjusted: {
      citations: getEffectiveOracleSection(citationResult, 'citations'),
      bibliography: getEffectiveOracleSection(bibliographyResult, 'bibliography'),
      divergenceSummary: mergeDivergenceSummaries(
        citationResult?.adjusted?.divergenceSummary,
        bibliographyResult?.adjusted?.divergenceSummary
      ),
    },
    citationsByType: citationResult?.citationsByType || {},
    componentSummary: bibliographyResult?.componentSummary || {},
    oracleSource: bibliographyResult?.oracleSource || citationResult?.oracleSource || 'citeproc-js',
    error: combinedError || null,
  };
}

/**
 * Compute fidelity score from oracle result
 */
function computeFidelityScore(oracleResult) {
  if (oracleResult.error) {
    return 0;
  }

  const citations = getEffectiveOracleSection(oracleResult, 'citations');
  const bibliography = getEffectiveOracleSection(oracleResult, 'bibliography');

  const citationsPassed = citations.passed || 0;
  const citationsTotal = citations.total || 0;
  const biblioPassed = bibliography.passed || 0;
  const biblioTotal = bibliography.total || 0;

  const totalPassed = citationsPassed + biblioPassed;
  const totalTests = citationsTotal + biblioTotal;

  return totalTests > 0 ? Math.min(1, totalPassed / totalTests) : 0;
}

/**
 * Load known divergences
 */
function loadDivergences() {
  try {
    const divergencePath = path.join(__dirname, 'report-data', 'known-divergences.json');
    const content = fs.readFileSync(divergencePath, 'utf8');
    return JSON.parse(content);
  } catch {
    return {};
  }
}

/**
 * Compute component match rate from oracle result
 */
function computeComponentMatchRate(oracleResult) {
  if (oracleResult.error || !oracleResult.bibliography) return null;

  let totalMatches = 0;
  let totalComponents = 0;

  for (const entry of oracleResult.bibliography.entries || []) {
    if (entry.match) {
      totalMatches += 11;
      totalComponents += 11;
    } else if (entry.components) {
      const matches = (entry.components.matches || []).length;
      const diffs = (entry.components.differences || []).length;
      totalMatches += matches;
      totalComponents += matches + diffs;
    }
  }

  return totalComponents > 0 ? parseFloat((totalMatches / totalComponents).toFixed(3)) : null;
}

function clamp(min, max, value) {
  return Math.max(min, Math.min(max, value));
}

function safePct(value) {
  return parseFloat(clamp(0, 100, value).toFixed(1));
}

function loadStyleYaml(styleName, stylePathOverride = null) {
  const stylePath = stylePathOverride || path.join(path.dirname(__dirname), 'styles', `${styleName}.yaml`);
  if (!fs.existsSync(stylePath)) {
    return {
      stylePath,
      rawStyleData: null,
      resolvedStyleData: null,
      error: `Style YAML not found: ${stylePath}`,
    };
  }
  try {
    const rawStyleData = yaml.load(fs.readFileSync(stylePath, 'utf8'), { schema: CUSTOM_TAG_SCHEMA });
    return {
      stylePath,
      rawStyleData,
      resolvedStyleData: resolveStyleData(rawStyleData),
      error: null,
    };
  } catch (error) {
    return {
      stylePath,
      rawStyleData: null,
      resolvedStyleData: null,
      error: `YAML parse error: ${error.message}`,
    };
  }
}

function flattenTemplateComponents(components) {
  const flattened = [];
  for (const component of components || []) {
    if (!component || typeof component !== 'object') continue;
    flattened.push(component);
    if (Array.isArray(component.items)) {
      flattened.push(...flattenTemplateComponents(component.items));
    }
  }
  return flattened;
}

function collectTemplateScopes(styleData) {
  const citation = styleData?.citation || {};
  const bibliography = styleData?.bibliography || {};
  const typeTemplates = bibliography['type-templates'] || {};
  const scopes = [];

  if (Array.isArray(citation.template)) {
    scopes.push({ name: 'citation.template', components: citation.template });
  }
  if (Array.isArray(citation.integral?.template)) {
    scopes.push({ name: 'citation.integral.template', components: citation.integral.template });
  }
  if (Array.isArray(citation['non-integral']?.template)) {
    scopes.push({ name: 'citation.non-integral.template', components: citation['non-integral'].template });
  }
  if (Array.isArray(bibliography.template)) {
    scopes.push({ name: 'bibliography.template', components: bibliography.template });
  }

  for (const [typeKey, template] of Object.entries(typeTemplates)) {
    if (Array.isArray(template)) {
      scopes.push({
        name: `bibliography.type-templates.${typeKey}`,
        components: template,
      });
    }
  }

  return scopes;
}

function parseOverrideKey(rawKey) {
  const key = String(rawKey || '').trim();
  if (!key) return [];
  if (key === 'default') return ['default'];
  return key
    .replace(/^\[/, '')
    .replace(/\]$/, '')
    .split(',')
    .map((part) => part.trim())
    .map((part) => part.replace(/^['"]|['"]$/g, ''))
    .filter(Boolean);
}

function resolveOverrideForType(overrides, refType) {
  if (!overrides || typeof overrides !== 'object') return null;
  let defaultOverride = null;
  for (const [rawKey, value] of Object.entries(overrides)) {
    const keys = parseOverrideKey(rawKey);
    if (keys.includes('default')) {
      defaultOverride = value;
      continue;
    }
    if (keys.includes(refType)) return value;
  }
  return defaultOverride;
}

function componentVisibleForType(component, refType) {
  const baseSuppressed = component?.suppress === true;
  const override = resolveOverrideForType(component?.overrides, refType);
  if (override && typeof override === 'object' && Object.prototype.hasOwnProperty.call(override, 'suppress')) {
    return override.suppress !== true;
  }
  return !baseSuppressed;
}

function isAnchorComponent(component) {
  return Boolean(component?.contributor || component?.title || component?.date);
}

function componentSemanticKey(component) {
  if (component.contributor) return `contributor:${component.contributor}`;
  if (component.title) return `title:${component.title}`;
  if (component.date) return `date:${component.date}:${component.form || 'default'}`;
  if (component.number) return `number:${component.number}`;
  if (component.variable) return `variable:${component.variable}`;
  if (component.items) return 'items-group';
  return Object.keys(component).sort().join('|') || 'unknown';
}

function countTemplatePresetUses(node) {
  let count = 0;
  function visit(value) {
    if (!value || typeof value !== 'object') return;
    if (Array.isArray(value)) {
      for (const item of value) visit(item);
      return;
    }
    for (const [key, child] of Object.entries(value)) {
      if (key === 'use-preset') count += 1;
      if (key === 'preset' && typeof child === 'string' && child.trim()) count += 1;
      visit(child);
    }
  }
  visit(node);
  return count;
}

function countOptionsPresetUses(styleData) {
  const optionScopes = [
    styleData?.options,
    styleData?.citation?.options,
    styleData?.bibliography?.options,
  ].filter(Boolean);

  const keys = ['processing', 'contributors', 'dates', 'titles', 'substitute'];
  let uses = 0;
  const fields = [];

  for (const options of optionScopes) {
    for (const key of keys) {
      const value = options[key];
      if (typeof value === 'string') {
        uses += 1;
        fields.push(key);
      } else if (value && typeof value === 'object') {
        if (typeof value.preset === 'string' && value.preset.trim()) {
          uses += 1;
          fields.push(key);
        } else if (typeof value['use-preset'] === 'string' && value['use-preset'].trim()) {
          uses += 1;
          fields.push(key);
        }
      }
    }
  }

  return { uses, fields };
}

function computeTypeCoverageScore(citationsByType) {
  const entries = Object.entries(citationsByType || {})
    .filter(([, stats]) => (stats?.total || 0) > 0);

  if (entries.length === 0) {
    return {
      score: 0,
      observedTypes: 0,
      averageTypePassRate: 0,
      breadthFactor: 0,
    };
  }

  const averageTypePassRate = entries
    .map(([, stats]) => stats.passed / stats.total)
    .reduce((sum, rate) => sum + rate, 0) / entries.length;
  const breadthFactor = clamp(0, 1, entries.length / 4);
  const score = ((averageTypePassRate * 0.7) + (breadthFactor * 0.3)) * 100;

  return {
    score: safePct(score),
    observedTypes: entries.length,
    averageTypePassRate: parseFloat((averageTypePassRate * 100).toFixed(1)),
    breadthFactor: parseFloat((breadthFactor * 100).toFixed(1)),
  };
}

function computeFallbackRobustness(styleData) {
  const bibliography = styleData?.bibliography || {};
  const typeTemplates = bibliography['type-templates'] || {};
  const typeTemplateSet = new Set(Object.keys(typeTemplates));
  const assessedTypes = CORE_FALLBACK_TYPES.filter((type) => !typeTemplateSet.has(type));
  const flattenedBase = flattenTemplateComponents(Array.isArray(bibliography.template) ? bibliography.template : []);

  if (assessedTypes.length === 0) {
    return {
      score: 100,
      assessedTypes: 0,
      passingTypes: 0,
      note: 'all core types have explicit type-templates',
    };
  }

  let passingTypes = 0;
  for (const refType of assessedTypes) {
    const visible = flattenedBase.filter((component) => componentVisibleForType(component, refType));
    const anchorCount = visible.filter(isAnchorComponent).length;
    if (visible.length > 0 && anchorCount >= 2) passingTypes += 1;
  }

  return {
    score: safePct((passingTypes / assessedTypes.length) * 100),
    assessedTypes: assessedTypes.length,
    passingTypes,
    note: 'base bibliography template only',
  };
}

function computeConcisionScore(styleData, format) {
  const scopes = collectTemplateScopes(styleData);
  const scopedComponents = scopes
    .map((scope) => ({
      name: scope.name,
      components: flattenTemplateComponents(scope.components),
    }))
    .filter((scope) => scope.components.length > 0);
  const flattened = scopedComponents.flatMap((scope) => scope.components);

  if (flattened.length === 0) {
    return {
      score: 0,
      totalComponents: 0,
      duplicates: 0,
      withinScopeDuplicates: 0,
      crossScopeRepeats: 0,
      overrideDensity: 0,
      targetComponents: 0,
    };
  }

  const semanticKeys = flattened.map(componentSemanticKey);
  let withinScopeDuplicates = 0;
  const keyScopeCount = new Map();

  for (const scope of scopedComponents) {
    const keys = scope.components.map(componentSemanticKey);
    const uniqueInScope = new Set(keys);
    withinScopeDuplicates += Math.max(0, keys.length - uniqueInScope.size);
    for (const key of uniqueInScope) {
      keyScopeCount.set(key, (keyScopeCount.get(key) || 0) + 1);
    }
  }

  let crossScopeRepeats = 0;
  for (const count of keyScopeCount.values()) {
    crossScopeRepeats += Math.max(0, count - 1);
  }

  const weightedDuplicates = withinScopeDuplicates + (crossScopeRepeats * 0.25);
  const duplicateRatio = weightedDuplicates / semanticKeys.length;
  const overrideCount = flattened.reduce(
    (sum, component) => sum + Object.keys(component.overrides || {}).length,
    0
  );
  const overrideDensity = overrideCount / flattened.length;
  const typeTemplateCoverage = Object.keys(styleData?.bibliography?.['type-templates'] || {})
    .reduce((sum, rawKey) => {
      const parsed = parseOverrideKey(rawKey).filter((key) => key !== 'default');
      return sum + (parsed.length || 1);
    }, 0);

  const componentTargets = {
    'author-date': 52,
    numeric: 55,
    note: 65,
  };
  const targetBase = componentTargets[format] || 55;
  const targetBonus = clamp(0, 35, Math.max(0, typeTemplateCoverage - 3) * 2.5);
  const target = targetBase + targetBonus;
  const componentPenalty = Math.max(0, flattened.length - target) * 0.9;
  const duplicatePenalty = duplicateRatio * 24;
  const overridePenalty = Math.max(0, overrideDensity - 1.5) * 12;
  const score = 100 - componentPenalty - duplicatePenalty - overridePenalty;

  return {
    score: safePct(score),
    totalComponents: flattened.length,
    duplicates: parseFloat(weightedDuplicates.toFixed(1)),
    withinScopeDuplicates,
    crossScopeRepeats,
    overrideDensity: parseFloat(overrideDensity.toFixed(2)),
    targetComponents: parseFloat(target.toFixed(1)),
  };
}

function computePresetUsageScore(styleData, concisionScore) {
  const templateUses = countTemplatePresetUses(styleData);
  const { uses: optionUses, fields: optionPresetFields } = countOptionsPresetUses(styleData);
  const weightedUses = (templateUses * 2) + optionUses;
  const uses = templateUses + optionUses;

  if (weightedUses >= 5) {
    return { score: 100, uses, templateUses, optionUses, weightedUses, optionPresetFields };
  }
  if (weightedUses >= 3) {
    return { score: 90, uses, templateUses, optionUses, weightedUses, optionPresetFields };
  }
  if (weightedUses >= 2) {
    return { score: 80, uses, templateUses, optionUses, weightedUses, optionPresetFields };
  }
  if (weightedUses >= 1) {
    return { score: 70, uses, templateUses, optionUses, weightedUses, optionPresetFields };
  }

  const baselineScore = concisionScore >= 80 ? 60 : 45;
  return {
    score: baselineScore,
    uses,
    templateUses,
    optionUses,
    weightedUses,
    optionPresetFields,
  };
}

function computeQualityMetrics(styleSpec, oracleResult, styleYamlPath = null) {
  const loaded = loadStyleYaml(styleSpec.name, styleYamlPath);
  if (!loaded.resolvedStyleData) {
    return {
      score: 0,
      error: loaded.error,
      subscores: {
        typeCoverage: { score: 0 },
        fallbackRobustness: { score: 0 },
        concision: { score: 0 },
        presetUsage: { score: 0 },
      },
    };
  }

  const authoredStyleData = loaded.rawStyleData || loaded.resolvedStyleData;
  const effectiveStyleData = loaded.resolvedStyleData;
  const typeCoverage = computeTypeCoverageScore(oracleResult.citationsByType || {});
  let fallbackRobustness = computeFallbackRobustness(effectiveStyleData);
  const concision = computeConcisionScore(effectiveStyleData, styleSpec.format);
  const presetUsage = computePresetUsageScore(authoredStyleData, concision.score);
  const weights = {
    typeCoverage: 0.35,
    fallbackRobustness: 0.25,
    concision: 0.25,
    presetUsage: 0.15,
  };

  // Citation-only note styles don't define bibliography templates, so
  // bibliography fallback robustness is not applicable.
  if (styleSpec.hasBibliography === false) {
    fallbackRobustness = {
      score: 100,
      assessedTypes: 0,
      passingTypes: 0,
      note: 'not applicable for citation-only style',
      notApplicable: true,
    };
    weights.fallbackRobustness = 0;
  }

  const weightSum = Object.values(weights).reduce((sum, value) => sum + value, 0);
  const rawScore = (
    (typeCoverage.score * weights.typeCoverage) +
    (fallbackRobustness.score * weights.fallbackRobustness) +
    (concision.score * weights.concision) +
    (presetUsage.score * weights.presetUsage)
  );
  const score = weightSum > 0 ? rawScore / weightSum : 0;

  return {
    score: safePct(score),
    error: null,
    subscores: {
      typeCoverage,
      fallbackRobustness,
      concision,
      presetUsage,
    },
  };
}

function buildPresentationFields(styleSpec, stylePolicy, sufficiencyPolicy) {
  const citationAuthority = resolveScopeAuthority(stylePolicy, 'citation');
  const bibliographyAuthority = resolveScopeAuthority(stylePolicy, 'bibliography');
  const effectiveScopes = getEffectiveVerificationScopes(stylePolicy, styleSpec.hasBibliography);
  const hasBibliographyScope = effectiveScopes.includes('bibliography');
  return {
    cslReach: styleSpec.cslReach,
    dependents: styleSpec.cslReach,
    impactPct: computeImpactPct(styleSpec.cslReach),
    originKey: styleSpec.originKey,
    originLabel: styleSpec.originLabel,
    originSortRank: styleSpec.originSortRank,
    benchmarkSource: normalizeBenchmarkSource(stylePolicy.authority),
    benchmarkAuthorityId: stylePolicy.authorityId,
    benchmarkLabel: formatAuthorityLabel(stylePolicy.authority, stylePolicy.authorityId),
    citationAuthorityLabel: formatAuthorityLabel(citationAuthority.authority, citationAuthority.authorityId),
    bibliographyAuthorityLabel: hasBibliographyScope
      ? formatAuthorityLabel(bibliographyAuthority.authority, bibliographyAuthority.authorityId)
      : null,
    hasScopedAuthorities: hasBibliographyScope && (
      citationAuthority.authority !== bibliographyAuthority.authority
      || citationAuthority.authorityId !== bibliographyAuthority.authorityId
    ),
    secondarySources: stylePolicy.secondary,
    secondarySourceLabels: (stylePolicy.secondary || []).map((authority) => formatAuthorityLabel(authority)),
    regressionBaseline: stylePolicy.regressionBaseline,
    regressionBaselineLabel: stylePolicy.regressionBaseline
      ? formatAuthorityLabel(stylePolicy.regressionBaseline)
      : null,
    verificationScopes: effectiveScopes,
    fixtureFamily: sufficiencyPolicy.family,
    defaultReportSufficient: sufficiencyPolicy.defaultReportSufficient,
    requiredReferenceTypes: sufficiencyPolicy.requiredReferenceTypes,
    requiredScenarios: sufficiencyPolicy.requiredScenarios,
    fixtureSets: sufficiencyPolicy.fixtureSets,
    benchmarkRuns: stylePolicy.benchmarkRuns || [],
    verificationNote: stylePolicy.note,
  };
}

function preflightSnapshots(coreStyles, verificationPolicy, stylesDir) {
  const issues = [];

  for (const styleSpec of coreStyles) {
    const stylePolicy = resolveVerificationPolicy(styleSpec.name, verificationPolicy);
    const citationAuthority = resolveScopeAuthority(stylePolicy, 'citation');
    if (citationAuthority.authority !== 'citeproc-js') continue;

    const stylePath = path.join(stylesDir, `${styleSpec.sourceName}.csl`);
    if (!fs.existsSync(stylePath)) continue;
    const citationsFixture = styleSpec.format === 'note' ? NOTE_CITATIONS_FIXTURE : DEFAULT_CITATIONS_FIXTURE;
    const snapshotStatus = getCslSnapshotStatus(stylePath, DEFAULT_REFS_FIXTURE, citationsFixture);
    if (!snapshotStatus.ok) {
      issues.push({
        style: styleSpec.name,
        sourceName: styleSpec.sourceName,
        status: snapshotStatus.status,
        message: snapshotStatus.message,
      });
    }
  }

  return issues.sort((left, right) => left.style.localeCompare(right.style));
}

async function processStyleReport(runtime, styleSpec, context) {
  const {
    divergences,
    stylesDir,
    styleYamlOverridePath,
    verificationPolicy,
    fixtureSufficiency,
    noteStyles,
  } = context;
  const stylePolicy = resolveVerificationPolicy(styleSpec.name, verificationPolicy);
  const sufficiencyPolicy = resolveFixtureSufficiency(stylePolicy.fixtureFamily, fixtureSufficiency);
  const primaryComparator = selectPrimaryComparator(styleSpec, stylePolicy);
  const citationAuthority = resolveScopeAuthority(stylePolicy, 'citation');
  const bibliographyAuthority = resolveScopeAuthority(stylePolicy, 'bibliography');

  if (primaryComparator === 'citum-baseline') {
    const oracleResult = await runNativeOracle(runtime, styleSpec.name);
    const fidelityScore = computeFidelityScore(oracleResult);
    const caseMismatches = collectCaseMismatchSummary(oracleResult);
    const bibliography = getEffectiveOracleSection(oracleResult, 'bibliography');
    const citations = getEffectiveOracleSection(oracleResult, 'citations');
    const rawBibliography = oracleResult.bibliography || { passed: 0, total: 0 };
    const rawCitations = oracleResult.citations || { passed: 0, total: 0 };
    const qualityMetrics = computeQualityMetrics(styleSpec, oracleResult, styleYamlOverridePath);
    const qualityScore = qualityMetrics.score / 100;
    let statusTier = 'failing';
    if (oracleResult.error) {
      statusTier = 'error';
    } else if (fidelityScore === 1.0) {
      statusTier = 'perfect';
    } else if (fidelityScore > 0) {
      statusTier = 'partial';
    }

    const notePositionAudit = styleSpec.format === 'note' && noteStyles.has(styleSpec.name)
      ? auditNoteStyle(noteStyles.get(styleSpec.name), { citumBin: runtime.citumBin })
      : null;

    return {
      styleRecord: {
        name: styleSpec.name,
        format: styleSpec.format,
        hasBibliography: styleSpec.hasBibliography,
        ...buildPresentationFields(styleSpec, stylePolicy, sufficiencyPolicy),
        fidelityScore: parseFloat(fidelityScore.toFixed(3)),
        citations,
        bibliography,
        rawCitations,
        rawBibliography,
        knownDivergences: divergences[styleSpec.name] || [],
        adjustedDivergences: oracleResult.adjusted?.divergenceSummary || {},
        caseMismatches,
        citationsByType: oracleResult.citationsByType || {},
        error: oracleResult.error || null,
        componentMatchRate: null,
        statusTier,
        componentSummary: {},
        citationEntries: null,
        oracleDetail: oracleResult.bibliography ? oracleResult.bibliography.entries : null,
        qualityScore: parseFloat(qualityScore.toFixed(3)),
        qualityBreakdown: qualityMetrics,
        oracleSource: 'citum-baseline',
        notePositionAudit,
      },
      errorCount: oracleResult.error ? 1 : 0,
      citations,
      bibliography,
      qualityScore,
    };
  }

  const stylePath = path.join(stylesDir, `${styleSpec.sourceName}.csl`);
  const styleYamlPath = styleYamlOverridePath || path.join(PROJECT_ROOT, 'styles', `${styleSpec.name}.yaml`);

  if (citationAuthority.authority === 'citeproc-js' && !fs.existsSync(stylePath)) {
    return {
      styleRecord: {
        name: styleSpec.name,
        format: styleSpec.format,
        hasBibliography: styleSpec.hasBibliography,
        ...buildPresentationFields(styleSpec, stylePolicy, sufficiencyPolicy),
        fidelityScore: 0,
        citations: { passed: 0, total: 0 },
        bibliography: { passed: 0, total: 0 },
        knownDivergences: divergences[styleSpec.name] || [],
        citationsByType: {},
        error: `Style file not found: ${stylePath}`,
        oracleDetail: null,
        qualityScore: 0,
        qualityBreakdown: null,
      },
      errorCount: 1,
      citations: { passed: 0, total: 0 },
      bibliography: { passed: 0, total: 0 },
      qualityScore: 0,
    };
  }

  let oracleResult;
  if (citationAuthority.authority === 'citeproc-js' && bibliographyAuthority.authority === 'biblatex') {
    const [citationResult, bibliographyResult] = await Promise.all([
      runCiteprocSnapshotOracle(runtime, stylePath, styleSpec.name, styleSpec.format),
      runBiblatexSnapshotOracle(runtime, styleSpec.name, styleYamlPath, bibliographyAuthority.authorityId),
    ]);
    oracleResult = composeScopedOracleResult(citationResult, bibliographyResult);
  } else if (primaryComparator === 'biblatex') {
    oracleResult = await runBiblatexSnapshotOracle(runtime, styleSpec.name, styleYamlPath, stylePolicy.authorityId);
  } else {
    oracleResult = await runCiteprocSnapshotOracle(runtime, stylePath, styleSpec.name, styleSpec.format);
  }

  const familySets = getAdditionalFixtureSetNames(sufficiencyPolicy.fixtureSets || []);
  if (citationAuthority.authority === 'citeproc-js' && fs.existsSync(stylePath)) {
    for (const setName of familySets) {
      const extra = await runFamilyFixtureOracle(runtime, stylePath, styleSpec.name, setName);
      if (!extra) continue;
      if (bibliographyAuthority.authority === 'citeproc-js') {
        mergeOracleResults(oracleResult, extra);
      } else {
        mergeCitationResults(oracleResult, extra);
      }
    }
  }

  const benchmarkRunResults = await executeBenchmarkRuns(stylePolicy.benchmarkRuns || [], async (benchmarkRun) =>
    runBenchmarkRun(runtime, styleSpec, stylePath, styleYamlPath, benchmarkRun)
  );
  for (const benchmarkRunResult of benchmarkRunResults) {
    mergeBenchmarkRunIntoOracle(oracleResult, benchmarkRunResult);
  }
  const benchmarkErrors = benchmarkRunResults
    .filter((benchmarkRunResult) => benchmarkRunResult.error)
    .map((benchmarkRunResult) => `benchmark ${benchmarkRunResult.id}: ${benchmarkRunResult.error}`);
  const publishedBenchmarkRunResults = benchmarkRunResults.map(toPublishedBenchmarkRunRecord);

  const fidelityScore = computeFidelityScore(oracleResult);
  const caseMismatches = collectCaseMismatchSummary(oracleResult);
  const citations = getEffectiveOracleSection(oracleResult, 'citations');
  const bibliography = getEffectiveOracleSection(oracleResult, 'bibliography');
  const rawCitations = oracleResult.citations || { passed: 0, total: 0 };
  const rawBibliography = oracleResult.bibliography || { passed: 0, total: 0 };
  const componentMatchRate = computeComponentMatchRate(oracleResult);
  const qualityMetrics = computeQualityMetrics(styleSpec, oracleResult, styleYamlPath);
  const qualityScore = qualityMetrics.score / 100;

  let statusTier = 'failing';
  const combinedStyleError = [oracleResult.error, ...benchmarkErrors].filter(Boolean).join('\n') || null;

  if (combinedStyleError) {
    statusTier = 'error';
  } else if (fidelityScore === 1.0) {
    statusTier = 'perfect';
  } else if (fidelityScore > 0) {
    statusTier = 'partial';
  }

  const notePositionAudit = styleSpec.format === 'note' && noteStyles.has(styleSpec.name)
    ? auditNoteStyle(noteStyles.get(styleSpec.name), { citumBin: runtime.citumBin })
    : null;

  const styleRecord = {
    name: styleSpec.name,
    format: styleSpec.format,
    hasBibliography: styleSpec.hasBibliography,
    ...buildPresentationFields(styleSpec, stylePolicy, sufficiencyPolicy),
    fidelityScore: parseFloat(fidelityScore.toFixed(3)),
    citations,
    bibliography,
    rawCitations,
    rawBibliography,
    knownDivergences: divergences[styleSpec.name] || [],
    adjustedDivergences: oracleResult.adjusted?.divergenceSummary || {},
    caseMismatches,
    citationsByType: oracleResult.citationsByType || {},
    error: combinedStyleError,
    componentMatchRate,
    statusTier,
    componentSummary: oracleResult.componentSummary || {},
    citationEntries: oracleResult.citations ? oracleResult.citations.entries : null,
    oracleDetail: oracleResult.bibliography ? oracleResult.bibliography.entries : null,
    benchmarkRunResults: publishedBenchmarkRunResults,
    qualityScore: parseFloat(qualityScore.toFixed(3)),
    qualityBreakdown: qualityMetrics,
    oracleSource: oracleResult.oracleSource || primaryComparator,
    styleYamlPath: toRepoRelativePath(styleYamlPath) || styleYamlPath,
    notePositionAudit,
  };
  styleRecord.richInputEvidence = buildRichInputEvidenceSummary(styleRecord, publishedBenchmarkRunResults);

  return {
    styleRecord,
    errorCount: combinedStyleError ? 1 : 0,
    citations,
    bibliography,
    qualityScore,
  };
}

/**
 * Generate compatibility report
 */
async function generateReport(options) {
  const stylesDir = getStylesDir(options.stylesDir);
  const provenanceConfig = loadReportProvenance();
  const discoveredStyles = discoverCoreStyles(provenanceConfig);
  const coreStyles = options.styleName
    ? discoveredStyles.filter((style) => style.name === options.styleName)
    : resolveSelectedStyles(discoveredStyles, options.styles);
  if (options.styleName && coreStyles.length === 0) {
    throw new Error(`Core style not found for --style: ${options.styleName}`);
  }
  const divergences = loadDivergences();
  const verificationPolicy = loadVerificationPolicy();
  const fixtureSufficiency = loadFixtureSufficiency();
  const generated = getTimestamp();
  const gitCommit = getGitCommit();
  const runtime = createReportRuntime(options);
  const preflightIssues = preflightSnapshots(coreStyles, verificationPolicy, stylesDir);
  const noteStyles = buildNoteStyleLookup();

  if (!runtime.allowLiveFallback && preflightIssues.length > 0) {
    for (const issue of preflightIssues) {
      process.stderr.write(`[snapshot ${issue.status}] ${issue.style}: ${issue.message}\n`);
    }
  }

  const styleJobs = await mapWithConcurrency(coreStyles, options.parallelism || DEFAULT_PARALLELISM, async (styleSpec) => {
    const result = await processStyleReport(runtime, styleSpec, {
      divergences,
      stylesDir,
      styleYamlOverridePath: options.styleFile,
      verificationPolicy,
      fixtureSufficiency,
      noteStyles,
    });
    if (result.styleRecord.error) {
      process.stderr.write(`Error processing ${styleSpec.name}: ${result.styleRecord.error}\n`);
    }
    return result;
  });

  const styles = styleJobs
    .map((job) => job.styleRecord)
    .sort((left, right) => left.name.localeCompare(right.name));
  let citationsTotal = 0;
  let citationsPassed = 0;
  let biblioTotal = 0;
  let biblioPassed = 0;
  let citationCaseMismatchTotal = 0;
  let bibliographyCaseMismatchTotal = 0;
  let qualityTotal = 0;
  let qualityCount = 0;
  let errorCount = 0;

  for (const job of styleJobs) {
    const citations = job.citations || { passed: 0, total: 0 };
    const bibliography = job.bibliography || { passed: 0, total: 0 };
    citationsTotal += citations.total || 0;
    citationsPassed += citations.passed || 0;
    biblioTotal += bibliography.total || 0;
    biblioPassed += bibliography.passed || 0;
    citationCaseMismatchTotal += job.styleRecord.caseMismatches?.citations || 0;
    bibliographyCaseMismatchTotal += job.styleRecord.caseMismatches?.bibliography || 0;
    qualityTotal += job.qualityScore || 0;
    qualityCount += 1;
    errorCount += job.errorCount || 0;
  }

  const knownDependents = coreStyles
    .filter((s) => typeof s.cslReach === 'number')
    .reduce((sum, s) => sum + s.cslReach, 0);
  const totalImpact = ((knownDependents / TOTAL_DEPENDENTS) * 100).toFixed(2);

  return {
    report: {
      generated,
      commit: gitCommit,
      source: 'scripts/report-core.js',
      metadata: {
        timestamp: generated,
        gitCommit,
        fixture: 'tests/fixtures/references-expanded.json',
        styleSelector: options.styleName
          ? `style:${options.styleName}`
          : options.styles?.length
            ? 'selected-styles'
            : 'core-styles',
        styleYamlOverride: options.styleFile ? toRepoRelativePath(options.styleFile) || options.styleFile : null,
        styles: coreStyles.map((style) => style.name),
        generator: 'scripts/report-core.js',
        richInputEvidence: {
          status: 'official-supplemental',
          headlineGate: 'baseline-fixtures',
        },
        extraFixtures: [
          'tests/fixtures/citations-note-expanded.json',
          'tests/fixtures/references-note-positions.json',
          'tests/fixtures/citations-note-positions.json',
        ],
        parallelism: options.parallelism || DEFAULT_PARALLELISM,
        cacheDir: runtime.cacheDir,
        preflight: {
          snapshotIssues: preflightIssues,
          allowLiveFallback: runtime.allowLiveFallback,
        },
        oracleComparison: {
          caseSensitive: runtime.caseSensitive,
        },
        ...(options.timings ? { timings: serializeTimingSummary(runtime) } : {}),
      },
      totalImpact: parseFloat(totalImpact),
      totalStyles: coreStyles.length,
      citationsOverall: { passed: citationsPassed, total: citationsTotal },
      bibliographyOverall: { passed: biblioPassed, total: biblioTotal },
      caseMismatchesOverall: {
        citations: citationCaseMismatchTotal,
        bibliography: bibliographyCaseMismatchTotal,
        total: citationCaseMismatchTotal + bibliographyCaseMismatchTotal,
      },
      qualityOverall: {
        score: qualityCount > 0 ? parseFloat((qualityTotal / qualityCount).toFixed(3)) : 0,
      },
      styles,
    },
    errorCount
  };
}

/**
 * Generate HTML dashboard
 */
function generateHtml(report) {
  const headerHtml = generateHtmlHeader(report);
  const statsHtml = generateHtmlStats(report);
  const sqiExplainerHtml = generateHtmlSqiExplainer();
  const tableHtml = generateHtmlTable(report);
  const footerHtml = generateHtmlFooter();

  return `${headerHtml}${statsHtml}${sqiExplainerHtml}${tableHtml}${footerHtml}`;
}

function generateHtmlHeader(report) {
  const generatedDate = new Date(report.generated).toUTCString();
  return `<!-- Auto-generated by report-core.js. Do not edit manually. -->
<!DOCTYPE html>
<html lang="en" class="scroll-smooth">

<head>
    <meta charset="utf-8" />
    <meta content="width=device-width, initial-scale=1.0" name="viewport" />
    <title>Citum | Style Compatibility Report</title>
    <meta name="description"
        content="Compatibility metrics for Citum against declared style authority sources.">

    <script src="https://cdn.tailwindcss.com?plugins=forms,container-queries,typography"></script>
    <link
        href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&amp;family=JetBrains+Mono:wght@400;500&amp;display=swap"
        rel="stylesheet" />
    <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet" />

    <script>
        tailwind.config = {
            darkMode: "class",
            theme: {
                extend: {
                    colors: {
                        "primary": "#2a94d6",
                        "background-light": "#fdfbf7",
                        "accent-cream": "#f5f2eb",
                    },
                    fontFamily: {
                        "display": ["Inter", "sans-serif"],
                        "mono": ["JetBrains Mono", "monospace"]
                    },
                    borderRadius: {
                        "DEFAULT": "0.25rem",
                        "lg": "0.5rem",
                        "xl": "0.75rem",
                        "full": "9999px"
                    },
                },
            },
        }
    </script>
    <style type="text/tailwindcss">
        body {
            font-family: 'Inter', sans-serif;
            color: #374151;
        }
        .font-mono {
            font-family: 'JetBrains Mono', monospace;
        }
        .glass-nav {
            background: rgba(253, 251, 247, 0.85);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid rgba(42, 148, 214, 0.1);
        }
        .accordion-toggle {
            cursor: pointer;
            user-select: none;
        }
        .accordion-content {
            display: none;
        }
        .accordion-content.active {
            display: table-row;
        }
        .badge-perfect {
            background-color: rgba(16, 185, 129, 0.1);
            color: #047857;
        }
        .badge-partial {
            background-color: rgba(251, 191, 36, 0.1);
            color: #92400e;
        }
        .badge-failing {
            background-color: rgba(239, 68, 68, 0.1);
            color: #7f1d1d;
        }
        .badge-pending {
            background-color: rgba(148, 163, 184, 0.1);
            color: #475569;
        }
    </style>
</head>

<body class="bg-background-light text-slate-700 selection:bg-primary/20">

    <!-- Navigation -->
    <nav class="fixed top-0 w-full z-50 glass-nav">
        <div class="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
            <div class="flex items-center gap-2 shrink-0">
                <a href="index.html" class="flex items-center gap-2 group">
                    <div class="w-8 h-8 bg-primary rounded flex items-center justify-center group-hover:brightness-110 transition-all">
                        <span class="text-white font-mono font-bold">C</span>
                    </div>
                    <span class="font-mono text-xl font-bold tracking-tight text-slate-900">Citum</span>
                </a>
            </div>
            <div class="hidden md:flex items-center gap-3 lg:gap-4 xl:gap-6 min-w-0 overflow-x-auto whitespace-nowrap pl-4">
                <a class="text-sm font-medium hover:text-primary transition-colors text-slate-600"
                    href="https://citum.org">Home</a>
                <a class="text-sm font-medium hover:text-primary transition-colors text-slate-600"
                    href="index.html">Docs</a>
                <a class="text-sm font-medium hover:text-primary transition-colors text-slate-600"
                    href="interactive-demo.html">Demo</a>
                <a class="text-sm font-medium hover:text-primary transition-colors text-slate-600"
                    href="examples.html">Examples</a>
                <a class="text-sm font-medium hover:text-primary transition-colors text-slate-600"
                    href="guides/style-author-guide.html">Style Guide</a>
                <a class="text-sm font-medium text-primary font-semibold"
                    href="reports.html">Reports</a>
                <a class="text-sm font-medium hover:text-primary transition-colors text-slate-600"
                    href="https://github.com/citum/citum-core">GitHub</a>
            </div>
            <button type="button"
                class="md:hidden inline-flex items-center justify-center rounded-lg border border-slate-200 bg-white/80 px-3 py-2 text-slate-700"
                data-nav-toggle aria-expanded="false" aria-controls="mobile-nav">
                <span class="material-icons text-[20px]">menu</span>
            </button>
        </div>
        <div id="mobile-nav" class="md:hidden hidden border-t border-slate-200 bg-background-light/95 px-6 py-4" data-mobile-menu>
            <div class="flex flex-col gap-3 text-sm font-medium text-slate-700">
                <a class="hover:text-primary transition-colors" href="https://citum.org">Home</a>
                <a class="hover:text-primary transition-colors" href="index.html">Docs</a>
                <a class="hover:text-primary transition-colors" href="interactive-demo.html">Demo</a>
                <a class="hover:text-primary transition-colors" href="examples.html">Examples</a>
                <a class="hover:text-primary transition-colors" href="guides/style-author-guide.html">Style Guide</a>
                <a class="text-primary font-semibold" href="reports.html">Reports</a>
                <a class="hover:text-primary transition-colors" href="https://github.com/citum/citum-core">GitHub</a>
            </div>
        </div>
    </nav>

    <!-- Header Section -->
    <header class="pt-24 pb-12 px-6 border-b border-slate-200">
        <div class="max-w-7xl mx-auto">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-4xl md:text-5xl font-mono font-bold tracking-tight text-slate-900 mb-2">
                        Style Compatibility Report
                    </h1>
                    <p class="text-slate-500">Compatibility metrics for styles in <code>styles/</code></p>
                </div>
            </div>
            <div class="flex flex-col sm:flex-row gap-4 items-start sm:items-center">
                <div class="text-sm text-slate-500 font-mono">Generated: ${generatedDate}</div>
                <div class="inline-flex items-center gap-2 px-3 py-1 rounded bg-slate-100 text-slate-700 text-xs font-mono border border-slate-200">
                    <span class="material-icons text-sm">code</span>
                    <span>${escapeHtml(report.commit)}</span>
                </div>
            </div>
            <div class="mt-5 flex flex-wrap gap-3 text-sm font-medium">
                <a class="text-primary hover:underline" href="compat.html">Compatibility</a>
                <a class="text-slate-600 hover:text-primary transition-colors" href="behavior-report.html">Engine Behavior Coverage</a>
                <a class="text-slate-600 hover:text-primary transition-colors" href="migration-behavior-report.html">Migration Behavior Coverage</a>
            </div>
        </div>
    </header>
`;
}

function generateHtmlStats(report) {
  const citationsPct = report.citationsOverall.total > 0
    ? ((report.citationsOverall.passed / report.citationsOverall.total) * 100).toFixed(1)
    : 0;
  const biblioPct = report.bibliographyOverall.total > 0
    ? ((report.bibliographyOverall.passed / report.bibliographyOverall.total) * 100).toFixed(1)
    : 0;
  const qualityPct = report.qualityOverall
    ? (report.qualityOverall.score * 100).toFixed(1)
    : '0.0';

  return `
    <!-- Statistics Cards -->
    <section class="py-12 px-6 bg-accent-cream">
        <div class="max-w-7xl mx-auto">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <!-- Core Styles -->
                <div class="bg-white rounded-xl border border-slate-200 p-6">
                    <div class="text-sm font-medium text-slate-500 mb-2">Core Styles</div>
                    <div class="text-3xl font-bold text-slate-900">${report.totalStyles}</div>
                    <div class="text-xs text-slate-400 mt-2">${report.totalImpact}% known CSL dependent coverage</div>
                </div>

                <!-- Citations Overall -->
                <div class="bg-white rounded-xl border border-slate-200 p-6">
                    <div class="text-sm font-medium text-slate-500 mb-2">Citations</div>
                    <div class="text-3xl font-bold text-slate-900">${report.citationsOverall.passed}/${report.citationsOverall.total}</div>
                    <div class="text-xs text-slate-400 mt-2">${citationsPct}% pass rate</div>
                </div>

                <!-- Bibliography Overall -->
                <div class="bg-white rounded-xl border border-slate-200 p-6">
                    <div class="text-sm font-medium text-slate-500 mb-2">Bibliography</div>
                    <div class="text-3xl font-bold text-slate-900">${report.bibliographyOverall.passed}/${report.bibliographyOverall.total}</div>
                    <div class="text-xs text-slate-400 mt-2">${biblioPct}% pass rate</div>
                </div>

                <!-- Quality Overall -->
                <div class="bg-white rounded-xl border border-slate-200 p-6">
                    <div class="text-sm font-medium text-slate-500 mb-2">Quality (SQI)</div>
                    <div class="text-3xl font-bold text-slate-900">${qualityPct}%</div>
                    <div class="text-xs text-slate-400 mt-2">Type coverage, fallback, concision, presets</div>
                </div>
            </div>
        </div>
    </section>
`;
}

function generateHtmlSqiExplainer() {
  return `
    <!-- SQI Explainer -->
    <section class="py-8 px-6">
        <div class="max-w-7xl mx-auto">
            <div class="bg-white rounded-xl border border-slate-200 p-6">
                <h2 class="text-lg font-semibold text-slate-900 mb-2">How To Read This Report</h2>
                <p class="text-sm text-slate-600 mb-3">
                    <strong>Fidelity</strong> is the hard gate: rendered output should match citeproc-js.
                    <strong>SQI</strong> (Style Quality Index) is secondary: it scores maintainability and fallback quality.
                </p>
                <p class="text-sm text-slate-600 mb-4">
                    Current working target for style waves is <code>&gt;=95% fidelity</code> and <code>&gt;=90 SQI</code>.
                    SQI should never be improved at the cost of fidelity.
                </p>
                <p class="text-sm text-slate-600 mb-4">
                    <strong>Lineage</strong> shows the source family a style derives from.
                    <strong>Authority</strong> shows the declared benchmark used for fidelity checks.
                    <strong>Regression baseline</strong> is listed separately when Citum snapshots are retained only as an internal guardrail.
                    <strong>CSL Reach</strong> is the count of dependent legacy CSL styles for comparable parents and is blank when there is no meaningful CSL analogue.
                </p>
                <a class="text-sm font-medium text-primary hover:underline" href="reference/SQI.md">
                    Read the full SQI definition and scoring details
                </a>
            </div>
        </div>
    </section>
`;
}

function generateHtmlTable(report) {
  let tableRows = '';

  for (const style of report.styles) {
    const fidelityPct = (style.fidelityScore * 100).toFixed(1);
    const qualityPct = ((style.qualityScore || 0) * 100).toFixed(1);
    const citationRate = style.citations.total > 0 ? style.citations.passed / style.citations.total : -1;
    const bibliographyRate = style.hasBibliography && style.bibliography.total > 0
      ? style.bibliography.passed / style.bibliography.total
      : -1;
    const cslReachValue = style.cslReach ?? -1;
    const componentRateValue = style.componentMatchRate !== null ? style.componentMatchRate : -1;

    const sqiScore = style.qualityBreakdown?.score ?? (style.qualityScore || 0) * 100;
    let sqiTier = 'D';
    let sqiTierRank = 1;
    let sqiBadgeClass = 'badge-failing';
    if (style.error || style.qualityBreakdown?.error) {
      sqiTier = 'ERR';
      sqiTierRank = 0;
      sqiBadgeClass = 'badge-pending';
    } else if (sqiScore >= 90) {
      sqiTier = 'A';
      sqiTierRank = 4;
      sqiBadgeClass = 'badge-perfect';
    } else if (sqiScore >= 80) {
      sqiTier = 'B';
      sqiTierRank = 3;
      sqiBadgeClass = 'bg-blue-100 text-blue-700';
    } else if (sqiScore >= 70) {
      sqiTier = 'C';
      sqiTierRank = 2;
      sqiBadgeClass = 'badge-partial';
    }

    const citationBadge = style.citations.total === 0
      ? 'badge-pending'
      : style.citations.passed === style.citations.total
        ? 'badge-perfect'
        : style.citations.passed > 0
          ? 'badge-partial'
          : 'badge-failing';

    const biblioBadge = !style.hasBibliography
      ? 'badge-pending'
      : style.bibliography.passed === style.bibliography.total && style.bibliography.total > 0
        ? 'badge-perfect'
        : style.bibliography.passed > 0
          ? 'badge-partial'
          : 'badge-failing';
    const biblioText = style.hasBibliography
      ? `${style.bibliography.passed}/${style.bibliography.total}`
      : 'N/A';

    let componentRateHtml = '—';
    if (style.componentMatchRate !== null) {
      const rate = style.componentMatchRate;
      const pct = (rate * 100).toFixed(0);
      let componentBadgeClass = 'bg-red-100 text-red-700';
      if (rate >= 0.9) {
        componentBadgeClass = 'bg-emerald-100 text-emerald-700';
      } else if (rate >= 0.7) {
        componentBadgeClass = 'bg-amber-100 text-amber-700';
      }
      componentRateHtml = `<span class="inline-flex items-center px-3 py-1 rounded text-xs font-medium ${componentBadgeClass}">${pct}%</span>`;
    }

    const toggleId = `toggle-${style.name}`;
    const contentId = `content-${style.name}`;
    tableRows += `
                <tr class="border-b border-slate-200 hover:bg-slate-50 accordion-toggle"
                    data-toggle="${toggleId}"
                    data-detail-id="${contentId}"
                    data-style-name="${escapeHtml(style.name.toLowerCase())}"
                    data-format="${escapeHtml(String(style.format).toLowerCase())}"
                    data-origin="${escapeHtml(String(style.originLabel || '').toLowerCase())}"
                    data-csl-reach="${cslReachValue}"
                    data-citation-rate="${citationRate}"
                    data-bibliography-rate="${bibliographyRate}"
                    data-component-rate="${componentRateValue}"
                    data-fidelity="${style.fidelityScore}"
                    data-quality="${style.qualityScore || 0}"
                    data-sqi-tier-rank="${sqiTierRank}">
                    <td class="px-6 py-4 text-sm font-medium text-slate-900">${style.name}</td>
                    <td class="hidden md:table-cell px-6 py-4 text-sm text-slate-600">${style.format}</td>
                    <td class="hidden md:table-cell px-6 py-4 text-sm text-slate-600">${escapeHtml(style.originLabel || '—')}</td>
                    <td class="hidden md:table-cell px-6 py-4 text-sm text-slate-500 font-mono">${escapeHtml(style.benchmarkLabel || '—')}</td>
                    <td class="hidden md:table-cell px-6 py-4 text-sm text-slate-600">${style.cslReach ?? '—'}</td>
                    <td class="hidden md:table-cell px-6 py-4">
                        <span class="inline-flex items-center px-3 py-1 rounded text-xs font-medium ${citationBadge}">
                            ${style.citations.passed}/${style.citations.total}
                        </span>
                    </td>
                    <td class="hidden md:table-cell px-6 py-4">
                        <span class="inline-flex items-center px-3 py-1 rounded text-xs font-medium ${biblioBadge}">
                            ${biblioText}
                        </span>
                    </td>
                    <td class="hidden md:table-cell px-6 py-4">
                        ${componentRateHtml}
                    </td>
                    <td class="px-6 py-4 text-sm font-mono text-slate-600">${fidelityPct}%</td>
                    <td class="hidden md:table-cell px-6 py-4 text-sm font-mono text-slate-600">${qualityPct}%</td>
                    <td class="px-6 py-4">
                        <span class="inline-flex items-center px-3 py-1 rounded text-xs font-medium ${sqiBadgeClass}">
                            ${sqiTier}
                        </span>
                    </td>
                    <td class="px-6 py-4 text-right">
                        <button class="text-slate-500 hover:text-primary text-xs font-medium transition-colors" onclick="toggleAccordion('${contentId}')">
                            <span class="material-icons text-base align-middle">expand_more</span>
                        </button>
                    </td>
                </tr>
                <tr class="accordion-content" id="${contentId}">
                    <td colspan="12" class="px-6 py-4 bg-slate-50">
                        <div class="max-w-4xl">
${generateDetailContent(style)}
                        </div>
                    </td>
                </tr>
    `;
  }

  return `
    <!-- Compatibility Table -->
    <section class="py-12 px-6">
        <div class="max-w-7xl mx-auto">
            <div class="rounded-xl border border-slate-200 overflow-hidden overflow-x-auto">
                <div class="px-6 py-4 bg-slate-50 border-b border-slate-200 flex flex-col sm:flex-row gap-3 sm:items-center sm:justify-between">
                    <label for="style-search" class="text-xs font-semibold text-slate-700">Search Styles</label>
                    <div class="flex items-center gap-3 w-full sm:w-auto">
                        <input
                            id="style-search"
                            type="search"
                            placeholder="Filter by style name..."
                            class="w-full sm:w-80 rounded-md border-slate-300 text-sm focus:border-primary focus:ring-primary"
                        />
                        <span id="style-search-count" class="text-xs text-slate-500 font-mono whitespace-nowrap"></span>
                    </div>
                </div>
                <table class="w-full">
                    <thead class="bg-slate-50 border-b border-slate-200">
                        <tr>
                            <th class="text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('style-name')">
                                    Style <span class="text-slate-400" id="sort-ind-style-name">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('format')">
                                    Format <span class="text-slate-400" id="sort-ind-format">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('origin')">
                                    Lineage <span class="text-slate-400" id="sort-ind-origin">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                Authority
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('csl-reach')">
                                    CSL Reach <span class="text-slate-400" id="sort-ind-csl-reach">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('citation-rate')">
                                    Citations <span class="text-slate-400" id="sort-ind-citation-rate">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('bibliography-rate')">
                                    Bibliography <span class="text-slate-400" id="sort-ind-bibliography-rate">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('component-rate')">
                                    Components <span class="text-slate-400" id="sort-ind-component-rate">↕</span>
                                </button>
                            </th>
                            <th class="text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('fidelity')">
                                    Fidelity <span class="text-slate-400" id="sort-ind-fidelity">↕</span>
                                </button>
                            </th>
                            <th class="hidden md:table-cell text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('quality')">
                                    Quality <span class="text-slate-400" id="sort-ind-quality">↕</span>
                                </button>
                            </th>
                            <th class="text-left px-6 py-4 text-xs font-semibold text-slate-700">
                                <button class="inline-flex items-center gap-1 hover:text-primary transition-colors" onclick="sortCompatTable('sqi-tier-rank')">
                                    SQI Tier <span class="text-slate-400" id="sort-ind-sqi-tier-rank">↕</span>
                                </button>
                            </th>
                            <th class="px-6 py-4"></th>
                        </tr>
                    </thead>
                    <tbody>
${tableRows}
                    </tbody>
                </table>
            </div>
        </div>
    </section>
`;
}

function getComparisonEntryTexts(entry) {
  return {
    benchmark: entry?.oracle ?? entry?.expected ?? '',
    citum: entry?.citum ?? entry?.actual ?? '',
  };
}

function generateDetailContent(style) {
  let html = '';
  const secondaryLabels = Array.isArray(style.secondarySourceLabels) && style.secondarySourceLabels.length > 0
    ? style.secondarySourceLabels.join(', ')
    : '—';
  const cslReachText = style.cslReach != null ? String(style.cslReach) : '—';

  html += `
                            <div class="mb-4 p-3 rounded border border-slate-200 bg-white">
                                <div class="text-xs font-semibold text-slate-900 mb-2">Verification Context</div>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                                    <div><span class="font-semibold text-slate-700">Lineage:</span> <span class="font-mono text-slate-600">${escapeHtml(style.originLabel || '—')}</span></div>
                                    <div><span class="font-semibold text-slate-700">Authority:</span> <span class="font-mono text-slate-600">${escapeHtml(style.benchmarkLabel || '—')}</span></div>
                                    ${style.hasScopedAuthorities ? `<div><span class="font-semibold text-slate-700">Citation authority:</span> <span class="font-mono text-slate-600">${escapeHtml(style.citationAuthorityLabel || '—')}</span></div>` : ''}
                                    ${style.hasScopedAuthorities ? `<div><span class="font-semibold text-slate-700">Bibliography authority:</span> <span class="font-mono text-slate-600">${escapeHtml(style.bibliographyAuthorityLabel || '—')}</span></div>` : ''}
                                    <div><span class="font-semibold text-slate-700">Secondary:</span> <span class="font-mono text-slate-600">${escapeHtml(secondaryLabels)}</span></div>
                                    ${style.regressionBaselineLabel ? `<div><span class="font-semibold text-slate-700">Regression baseline:</span> <span class="font-mono text-slate-600">${escapeHtml(style.regressionBaselineLabel)}</span></div>` : ''}
                                    <div><span class="font-semibold text-slate-700">CSL Reach:</span> <span class="font-mono text-slate-600">${escapeHtml(cslReachText)}</span></div>
                                </div>
                                ${style.verificationNote ? `<div class="mt-3 text-xs text-slate-600"><strong>Note:</strong> ${escapeHtml(style.verificationNote)}</div>` : ''}
                            </div>
`;

  if (style.error) {
    html += `
                            <div class="p-4 rounded-lg bg-red-50 border border-red-200 mb-4">
                                <div class="text-sm font-medium text-red-700 mb-1">Error</div>
                                <div class="text-xs text-red-600 font-mono">${escapeHtml(style.error)}</div>
                            </div>
`;
  }

  if (style.qualityBreakdown) {
    const qb = style.qualityBreakdown;
    const overall = ((style.qualityScore || 0) * 100).toFixed(1);
    const typeCoverage = qb.subscores?.typeCoverage?.score ?? 0;
    const fallback = qb.subscores?.fallbackRobustness?.score ?? 0;
    const concision = qb.subscores?.concision?.score ?? 0;
    const presets = qb.subscores?.presetUsage?.score ?? 0;
    html += `
                            <div class="mb-4 p-3 rounded border border-slate-200 bg-white">
                                <div class="text-xs font-semibold text-slate-900 mb-2">Quality (SQI): ${overall}%</div>
                                <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs font-mono">
                                    <div class="px-2 py-1 rounded bg-slate-100 text-slate-700">type ${typeCoverage.toFixed(1)}%</div>
                                    <div class="px-2 py-1 rounded bg-slate-100 text-slate-700">fallback ${fallback.toFixed(1)}%</div>
                                    <div class="px-2 py-1 rounded bg-slate-100 text-slate-700">concision ${concision.toFixed(1)}%</div>
                                    <div class="px-2 py-1 rounded bg-slate-100 text-slate-700">presets ${presets.toFixed(1)}%</div>
                                </div>
                            </div>
`;
  }

  if (style.notePositionAudit) {
    const audit = style.notePositionAudit;
    const regression = audit.regression || { status: audit.status, issues: audit.issues || [], profile: audit.profile };
    const hasConformance = Boolean(audit.conformance);
    const conformance = audit.conformance || { status: 'not-evaluated', issues: [], family: 'n/a', unresolved: [] };
    const regressionBadgeClass = regression.status === 'pass'
      ? 'bg-emerald-100 text-emerald-700'
      : regression.status === 'configuration-gap'
        ? 'bg-amber-100 text-amber-700'
        : 'bg-red-100 text-red-700';
    const conformanceBadgeClass = conformance.status === 'pass'
      ? 'bg-sky-100 text-sky-700'
      : conformance.status === 'not-evaluated'
        ? 'bg-slate-100 text-slate-700'
      : 'bg-amber-100 text-amber-700';
    const regressionIssueText = regression.issues.length === 0
      ? '<span class="text-xs text-slate-500">No repeated-note issues detected.</span>'
      : regression.issues
        .map((issue) => `<div class="text-xs font-mono text-slate-600">${escapeHtml(issue.kind)}: ${escapeHtml(issue.message)}</div>`)
        .join('');
    const conformanceIssueText = !hasConformance
      ? '<span class="text-xs text-slate-500">Normative conformance was not evaluated for this report artifact.</span>'
      : conformance.issues.length === 0
      ? '<span class="text-xs text-slate-500">No settled conformance gaps detected.</span>'
      : conformance.issues
        .map((issue) => `<div class="text-xs font-mono text-slate-600">${escapeHtml(issue.kind)}: ${escapeHtml(issue.message)}</div>`)
        .join('');
    const unresolvedText = !hasConformance
      ? '<span class="text-xs text-slate-500">No conformance metadata available.</span>'
      : Array.isArray(conformance.unresolved) && conformance.unresolved.length > 0
      ? `<div class="text-xs text-slate-500">Unresolved: ${escapeHtml(conformance.unresolved.join(', '))}</div>`
      : '<span class="text-xs text-slate-500">No unresolved conformance dimensions recorded.</span>';
    html += `
                            <div class="mb-4 p-3 rounded border border-slate-200 bg-white">
                                <div class="text-xs font-semibold text-slate-900 mb-3">Repeated-Note Audit</div>
                                <div class="mb-3">
                                    <div class="flex items-center justify-between gap-3 mb-2">
                                        <div class="text-xs font-semibold text-slate-700">Regression Layer</div>
                                        <span class="inline-flex items-center px-2 py-1 rounded text-xs font-medium ${regressionBadgeClass}">
                                            ${escapeHtml(regression.status)}
                                        </span>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs mb-2">
                                        <div><span class="font-semibold text-slate-700">Profile:</span> <span class="font-mono text-slate-600">${escapeHtml(regression.profile || '—')}</span></div>
                                        <div><span class="font-semibold text-slate-700">Issues:</span> <span class="font-mono text-slate-600">${regression.issues.length}</span></div>
                                    </div>
                                    ${regressionIssueText}
                                </div>
                                <div>
                                    <div class="flex items-center justify-between gap-3 mb-2">
                                        <div class="text-xs font-semibold text-slate-700">Normative Conformance</div>
                                        <span class="inline-flex items-center px-2 py-1 rounded text-xs font-medium ${conformanceBadgeClass}">
                                            ${escapeHtml(conformance.status)}
                                        </span>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs mb-2">
                                        <div><span class="font-semibold text-slate-700">Family:</span> <span class="font-mono text-slate-600">${escapeHtml(conformance.family || '—')}</span></div>
                                        <div><span class="font-semibold text-slate-700">Issues:</span> <span class="font-mono text-slate-600">${conformance.issues.length}</span></div>
                                    </div>
                                    ${conformanceIssueText}
                                    <div class="mt-2">
                                        ${unresolvedText}
                                    </div>
                                </div>
                            </div>
`;
  }

  if (Array.isArray(style.benchmarkRunResults) && style.benchmarkRunResults.length > 0) {
    html += `
                            <div class="mb-4 p-3 rounded border border-slate-200 bg-white">
                                <div class="text-xs font-semibold text-slate-900 mb-1">Official Supplemental Rich Benchmark Evidence</div>
                                <div class="text-xs text-slate-500 mb-3">Baseline fidelity remains the gate. Rich benchmark runs extend the official evidence for configured styles.</div>
                                <div class="space-y-3">
`;
    for (const benchmarkRun of style.benchmarkRunResults) {
      const statusClass = benchmarkRun.status === 'pass'
        ? 'bg-emerald-100 text-emerald-700'
        : benchmarkRun.status === 'ok'
          ? 'bg-slate-100 text-slate-600'
          : 'bg-red-100 text-red-700';
      const scopeText = benchmarkRun.scope === 'both' ? 'citation + bibliography' : benchmarkRun.scope;
      const contributionText = benchmarkRun.countTowardFidelity ? 'counts toward fidelity' : 'diagnostic only';
      const thresholdText = benchmarkRun.minPassRate != null
        ? `${(benchmarkRun.minPassRate * 100).toFixed(0)}%`
        : 'none';
      const bibliographyText = benchmarkRun.bibliography
        ? `${benchmarkRun.bibliography.passed}/${benchmarkRun.bibliography.total}`
        : benchmarkRun.bibliographyEntries != null
          ? `${benchmarkRun.bibliographyEntries} rendered`
          : '—';
      const citationsText = benchmarkRun.citations
        ? `${benchmarkRun.citations.passed}/${benchmarkRun.citations.total}`
        : '—';

      html += `
                                    <div class="rounded border border-slate-200 p-3">
                                        <div class="flex items-center justify-between gap-3 mb-2">
                                            <div class="text-xs font-semibold text-slate-800">${escapeHtml(benchmarkRun.label)}</div>
                                            <span class="inline-flex items-center px-2 py-1 rounded text-xs font-medium ${statusClass}">
                                                ${escapeHtml(benchmarkRun.status)}
                                            </span>
                                        </div>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                                            <div><span class="font-semibold text-slate-700">Runner:</span> <span class="font-mono text-slate-600">${escapeHtml(benchmarkRun.runner)}</span></div>
                                            <div><span class="font-semibold text-slate-700">Scope:</span> <span class="font-mono text-slate-600">${escapeHtml(scopeText)}</span></div>
                                            <div><span class="font-semibold text-slate-700">Contribution:</span> <span class="font-mono text-slate-600">${escapeHtml(contributionText)}</span></div>
                                            <div><span class="font-semibold text-slate-700">Threshold:</span> <span class="font-mono text-slate-600">${escapeHtml(thresholdText)}</span></div>
                                            <div><span class="font-semibold text-slate-700">Bibliography:</span> <span class="font-mono text-slate-600">${escapeHtml(bibliographyText)}</span></div>
                                            ${benchmarkRun.citations ? `<div><span class="font-semibold text-slate-700">Citations:</span> <span class="font-mono text-slate-600">${escapeHtml(citationsText)}</span></div>` : ''}
                                            <div><span class="font-semibold text-slate-700">Refs fixture:</span> <span class="font-mono text-slate-600">${escapeHtml(benchmarkRun.refsFixture)}</span></div>
                                        </div>
                                        ${benchmarkRun.error ? `<div class="mt-2 text-xs font-mono text-red-600">${escapeHtml(benchmarkRun.error)}</div>` : ''}
                                    </div>
`;
    }
    html += `
                                </div>
                            </div>
`;
  }

  if (style.componentSummary && Object.keys(style.componentSummary).length > 0) {
    const issues = Object.entries(style.componentSummary)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8);
    html += `
                            <div class="mb-4">
                                <div class="text-xs font-semibold text-slate-700 mb-2">Top Component Issues</div>
                                <div class="flex flex-wrap gap-2">
`;
    for (const [issue, count] of issues) {
      html += `
                                    <span class="px-2 py-1 rounded bg-slate-100 text-slate-600 text-xs font-mono">
                                        ${escapeHtml(issue)} <span class="font-bold">×${count}</span>
                                    </span>
`;
    }
    html += `
                                </div>
                            </div>
`;
  }

  if (style.citationsByType && Object.keys(style.citationsByType).length > 0) {
    const citationTypes = Object.entries(style.citationsByType)
      .sort((a, b) => {
        const aPct = a[1].total > 0 ? (a[1].passed / a[1].total) : 0;
        const bPct = b[1].total > 0 ? (b[1].passed / b[1].total) : 0;
        if (aPct !== bPct) return aPct - bPct;
        return a[0].localeCompare(b[0]);
      });
    html += `
                            <div class="mb-4">
                                <div class="text-xs font-semibold text-slate-900 mb-2">Citation Type Coverage (${citationTypes.length} types)</div>
                                <div class="flex flex-wrap gap-2">
`;
    for (const [type, stats] of citationTypes) {
      const pct = stats.total > 0 ? Math.round((stats.passed / stats.total) * 100) : 0;
      let badgeClass = 'bg-red-100 text-red-700';
      if (pct === 100) {
        badgeClass = 'bg-emerald-100 text-emerald-700';
      } else if (pct >= 70) {
        badgeClass = 'bg-amber-100 text-amber-700';
      }
      html += `
                                    <span class="px-2 py-1 rounded ${badgeClass} text-xs font-mono">
                                        ${escapeHtml(type)} ${stats.passed}/${stats.total}
                                    </span>
`;
    }
    html += `
                                </div>
                            </div>
`;
  }

  if (style.citationEntries && style.citationEntries.length > 0) {
    const failedEntries = style.citationEntries.filter(e => !e.match);
    if (failedEntries.length === 0) {
      html += `
                            <div class="mb-4 p-3 rounded bg-emerald-50 border border-emerald-200">
                                <div class="text-xs font-semibold text-emerald-700">All ${style.citationEntries.length} citations match ✓</div>
                            </div>
`;
    } else {
      html += `
                            <div class="mb-4">
                                <div class="text-xs font-semibold text-slate-900 mb-2">Failed Citations (${failedEntries.length}/${style.citationEntries.length})</div>
                                <div class="overflow-x-auto">
                                    <table class="w-full text-xs border-collapse">
                                        <thead>
                                            <tr class="border-b border-slate-300 bg-slate-100">
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">#</th>
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">Benchmark</th>
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">Citum</th>
                                                <th class="text-center px-2 py-1 font-medium text-slate-700">Match</th>
                                            </tr>
                                        </thead>
                                        <tbody>
`;
      for (const entry of failedEntries) {
        const texts = getComparisonEntryTexts(entry);
        const benchmarkText = texts.benchmark ? texts.benchmark.substring(0, 100) : '(empty)';
        const citumText = texts.citum ? texts.citum.substring(0, 100) : '(empty)';
        html += `
                                            <tr class="border-b border-slate-200 hover:bg-slate-50">
                                                <td class="px-2 py-1 text-slate-600">${escapeHtml(entry.id)}</td>
                                                <td class="px-2 py-1 font-mono text-slate-600 text-xs" title="${escapeHtml(texts.benchmark)}">${escapeHtml(benchmarkText)}</td>
                                                <td class="px-2 py-1 font-mono text-slate-600 text-xs" title="${escapeHtml(texts.citum)}">${escapeHtml(citumText)}</td>
                                                <td class="px-2 py-1 text-center font-bold text-red-600">✗</td>
                                            </tr>
`;
      }
      html += `
                                        </tbody>
                                    </table>
                                </div>
                            </div>
`;
    }
  }

  if (style.oracleDetail && style.oracleDetail.length > 0) {
    html += `
                            <div class="mt-4">
                                <div class="text-xs font-semibold text-slate-900 mb-2">Bibliography Entries (${style.oracleDetail.length})</div>
                                <div class="overflow-x-auto">
                                    <table class="w-full text-xs border-collapse">
                                        <thead>
                                            <tr class="border-b border-slate-300 bg-slate-100">
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">#</th>
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">Benchmark</th>
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">Citum</th>
                                                <th class="text-center px-2 py-1 font-medium text-slate-700">Match</th>
                                                <th class="text-left px-2 py-1 font-medium text-slate-700">Issues</th>
                                            </tr>
                                        </thead>
                                        <tbody>
`;

    for (let i = 0; i < style.oracleDetail.length; i++) {
      const entry = style.oracleDetail[i];
      const matchIcon = entry.match === true ? '✓' : entry.match === false ? '✗' : '–';
      const matchColor = entry.match === true ? 'text-emerald-600' : entry.match === false ? 'text-red-600' : 'text-slate-400';
      const texts = getComparisonEntryTexts(entry);
      const benchmarkText = texts.benchmark ? texts.benchmark.substring(0, 100) : '(empty)';
      const citumText = texts.citum ? texts.citum.substring(0, 100) : '(empty)';

      let issuesText = '—';
      if (!entry.match) {
        if (entry.issues && entry.issues.length > 0) {
          issuesText = entry.issues
            .map(iss => iss.component ? `${iss.component}:${iss.issue}` : iss.issue)
            .join(', ');
        }
      }

      html += `
                                            <tr class="border-b border-slate-200 hover:bg-slate-50">
                                                <td class="px-2 py-1 text-slate-600">${i + 1}</td>
                                                <td class="px-2 py-1 font-mono text-slate-600 text-xs" title="${escapeHtml(texts.benchmark)}">${escapeHtml(benchmarkText)}</td>
                                                <td class="px-2 py-1 font-mono text-slate-600 text-xs" title="${escapeHtml(texts.citum)}">${escapeHtml(citumText)}</td>
                                                <td class="px-2 py-1 text-center font-bold ${matchColor}">${matchIcon}</td>
                                                <td class="px-2 py-1 text-slate-600 text-xs font-mono">${escapeHtml(issuesText)}</td>
                                            </tr>
`;
    }

    html += `
                                        </tbody>
                                    </table>
                                </div>
                            </div>
`;
  }

  if (style.knownDivergences && style.knownDivergences.length > 0) {
    html += `
                            <div class="p-4 rounded-lg bg-primary/5 border border-primary/20 mt-4">
                                <div class="text-sm font-semibold text-primary mb-2">Citum Extensions</div>
`;
    for (const divergence of style.knownDivergences) {
      html += `
                                <div class="text-xs text-slate-700 mb-2">
                                    <strong>${escapeHtml(divergence.feature)}:</strong> ${escapeHtml(divergence.description)}
                                </div>
`;
    }
    html += `
                            </div>
`;
  }

  return html;
}

function generateHtmlFooter() {
  return `

    <!-- Footer -->
    <footer class="py-12 px-6 border-t border-slate-200 bg-white">
        <div class="max-w-7xl mx-auto">
            <div class="flex flex-col md:flex-row justify-between items-center gap-8">
                <div class="flex items-center gap-2">
                    <a href="index.html" class="flex items-center gap-2 group">
                        <div class="w-6 h-6 bg-primary rounded flex items-center justify-center group-hover:brightness-110 transition-all">
                            <span class="text-white font-mono text-xs font-bold">C</span>
                        </div>
                        <span class="font-mono text-lg font-bold text-slate-900">Citum</span>
                    </a>
                </div>
                <div class="flex gap-8 text-sm font-medium text-slate-500">
                    <a class="hover:text-primary transition-colors" href="https://github.com/citum/citum-core">GitHub</a>
                    <a class="hover:text-primary transition-colors" href="index.html">Docs</a>
                    <a class="hover:text-primary transition-colors" href="examples.html">Examples</a>
                    <a class="hover:text-primary transition-colors" href="reports.html">Reports</a>
                </div>
                <div class="text-sm text-slate-400">
                    © 2026 Citum Project. MIT Licensed.
                </div>
            </div>
        </div>
    </footer>

    <script>
        const navToggle = document.querySelector("[data-nav-toggle]");
        const mobileMenu = document.querySelector("[data-mobile-menu]");
        if (navToggle && mobileMenu) {
            navToggle.addEventListener("click", () => {
                const expanded = navToggle.getAttribute("aria-expanded") === "true";
                navToggle.setAttribute("aria-expanded", String(!expanded));
                mobileMenu.classList.toggle("hidden", expanded);
            });
        }

        const sortState = { key: null, direction: 1 };
        const filterState = { query: '' };

        function toggleAccordion(contentId) {
            const content = document.getElementById(contentId);
            if (content) content.classList.toggle('active');
        }

        function updateSortIndicators(activeKey, direction) {
            document.querySelectorAll('[id^="sort-ind-"]').forEach((el) => {
                el.textContent = '↕';
                el.classList.remove('text-primary');
                el.classList.add('text-slate-400');
            });
            const active = document.getElementById('sort-ind-' + activeKey);
            if (active) {
                active.textContent = direction > 0 ? '↑' : '↓';
                active.classList.remove('text-slate-400');
                active.classList.add('text-primary');
            }
        }

        function sortCompatTable(key) {
            const tbody = document.querySelector('table tbody');
            if (!tbody) return;

            const summaryRows = Array.from(tbody.querySelectorAll('tr.accordion-toggle'));
            const rowPairs = summaryRows.map((summary) => {
                const detailId = summary.dataset.detailId;
                const detail = detailId ? document.getElementById(detailId) : null;
                return { summary, detail };
            });

            const defaultAsc = key === 'style-name' || key === 'format' || key === 'origin';
            if (sortState.key === key) {
                sortState.direction *= -1;
            } else {
                sortState.key = key;
                sortState.direction = defaultAsc ? 1 : -1;
            }

            const asNumber = (value) => {
                const parsed = Number(value);
                return Number.isNaN(parsed) ? -Infinity : parsed;
            };
            const asText = (value) => String(value || '').toLowerCase();

            // Convert kebab-case key to camelCase for dataset access
            const datasetKey = key.replace(/-([a-z])/g, (g) => g[1].toUpperCase());

            rowPairs.sort((a, b) => {
                const left = a.summary.dataset[datasetKey] || '';
                const right = b.summary.dataset[datasetKey] || '';
                const numericKeys = new Set([
                    'csl-reach',
                    'citation-rate',
                    'bibliography-rate',
                    'component-rate',
                    'fidelity',
                    'quality',
                    'sqi-tier-rank',
                ]);

                if (key === 'csl-reach') {
                    const leftUnknown = left === '' || Number(left) < 0;
                    const rightUnknown = right === '' || Number(right) < 0;
                    if (leftUnknown !== rightUnknown) {
                        return leftUnknown ? 1 : -1;
                    }
                }

                if (numericKeys.has(key)) {
                    return (asNumber(left) - asNumber(right)) * sortState.direction;
                }
                return asText(left).localeCompare(asText(right)) * sortState.direction;
            });

            for (const pair of rowPairs) {
                tbody.appendChild(pair.summary);
                if (pair.detail) tbody.appendChild(pair.detail);
            }

            updateSortIndicators(key, sortState.direction);
        }

        function updateFilterCount(visible, total) {
            const count = document.getElementById('style-search-count');
            if (!count) return;
            count.textContent = visible === total
                ? total + ' styles'
                : visible + ' of ' + total + ' styles';
        }

        function applyStyleFilter() {
            const tbody = document.querySelector('table tbody');
            if (!tbody) return;

            const summaryRows = Array.from(tbody.querySelectorAll('tr.accordion-toggle'));
            const query = filterState.query;
            let visible = 0;

            for (const summary of summaryRows) {
                const detailId = summary.dataset.detailId;
                const detail = detailId ? document.getElementById(detailId) : null;
                const haystack = (summary.dataset.styleName || '').toLowerCase();
                const isMatch = !query || haystack.includes(query);

                summary.style.display = isMatch ? '' : 'none';
                if (detail) {
                    detail.style.display = isMatch ? '' : 'none';
                    if (!isMatch) detail.classList.remove('active');
                }
                if (isMatch) visible += 1;
            }

            updateFilterCount(visible, summaryRows.length);
        }

        function initStyleSearch() {
            const input = document.getElementById('style-search');
            if (!input) return;
            input.addEventListener('input', (event) => {
                filterState.query = String(event.target.value || '').trim().toLowerCase();
                applyStyleFilter();
            });

            const tbody = document.querySelector('table tbody');
            if (tbody) {
                tbody.addEventListener('click', (event) => {
                    const row = event.target.closest('tr.accordion-toggle');
                    if (!row) return;
                    // Don't re-toggle if the expand button itself was clicked (it has its own onclick)
                    if (event.target.closest('button')) return;
                    const detailId = row.dataset.detailId;
                    if (detailId) toggleAccordion(detailId);
                });
            }

            applyStyleFilter();
        }

        document.addEventListener('DOMContentLoaded', initStyleSearch);
    </script>

</body>

</html>
`;
}

function escapeHtml(text) {
  if (!text) return '';
  return String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

/**
 * Main entry point
 */
async function main() {
  try {
    const options = parseArgs();
    const stylesDir = options.stylesDir || path.join(PROJECT_ROOT, 'styles-legacy');
    const datasetMessage = maybeDatasetErrorForFile(stylesDir, 'report-core.js');
    if (datasetMessage) {
      throw new Error(datasetMessage);
    }
    const { report, errorCount } = await generateReport(options);

    // Output JSON to stdout
    console.log(JSON.stringify(report, null, 2));

    // Generate and write HTML if requested
    if (options.writeHtml) {
      const htmlPath = options.outputHtml || path.join(path.dirname(__dirname), 'docs', 'compat.html');
      const htmlDir = path.dirname(htmlPath);

      if (!fs.existsSync(htmlDir)) {
        fs.mkdirSync(htmlDir, { recursive: true });
      }

      const htmlContent = generateHtml(report);
      fs.writeFileSync(htmlPath, htmlContent, 'utf8');
      process.stderr.write(`HTML report written to: ${htmlPath}\n`);
    }

    if (errorCount > 0) {
      process.stderr.write(`\nTotal styles with errors: ${errorCount}\n`);
      process.exit(1);
    }
  } catch (error) {
    process.stderr.write(`Error: ${error.message}\n`);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  buildNoteStyleLookup,
  computeFidelityScore,
  createReportRuntime,
  discoverCoreStyles,
  equivalentText,
  expandCompoundBibEntries,
  formatAuthorityLabel,
  getEffectiveOracleSection,
  getCslSnapshotStatus,
  generateHtml,
  generateReport,
  getComparisonEntryTexts,
  mapWithConcurrency,
  normalizeBenchmarkSource,
  parseArgs,
  preflightSnapshots,
  resolveSelectedStyles,
  runCachedJsonJob,
  buildEmptyOracleResult,
  cloneOracleResult,
  executeBenchmarkRuns,
  mergeBenchmarkRunIntoOracle,
  mergeOracleResults,
  toPublishedBenchmarkRunRecord,
  selectPrimaryComparator,
  serializeTimingSummary,
  textSimilarity,
  mergeDivergenceSummaries,
};
