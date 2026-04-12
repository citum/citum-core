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
  getFixtureFiles,
  inferLegacySourceName,
  isProjectStylePath,
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
  const embeddedRoot = path.join(stylesRoot, 'embedded');
  if (!fs.existsSync(stylesRoot)) {
    throw new Error(`Core styles directory not found: ${stylesRoot}`);
  }

  const findStyles = (dir) => {
    if (!fs.existsSync(dir)) return [];
    return fs.readdirSync(dir)
      .filter((entry) => entry.endsWith('.yaml'))
      .map((filename) => ({
        filename,
        stylePath: path.join(dir, filename),
        name: path.basename(filename, '.yaml'),
      }));
  };

  const allStyles = [
    ...findStyles(stylesRoot),
    ...findStyles(embeddedRoot),
  ].filter((style) => !SKIPPED_STYLES.includes(style.name))
    .sort((a, b) => a.name.localeCompare(b.name));

  if (allStyles.length === 0) {
    throw new Error(`No style YAML files found in: ${stylesRoot}`);
  }

  return allStyles.map(({ stylePath, name }) => {
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
      path: stylePath,
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
  const hasPreset = typeof bibliography['use-preset'] === 'string' && bibliography['use-preset'].trim().length > 0;
  const typeTemplates = bibliography['type-templates'];
  const hasTypeTemplates = Boolean(typeTemplates && Object.keys(typeTemplates).length > 0);
  const typeVariants = bibliography['type-variants'];
  const hasTypeVariants = Boolean(typeVariants && Object.keys(typeVariants).length > 0);
  return hasTemplate || hasPreset || hasTypeTemplates || hasTypeVariants;
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

function determineBenchmarkStatus(oracleResult, minPassRate) {
  if (oracleResult.error) return 'error';
  if (minPassRate != null) {
    const bib = oracleResult.bibliography || { passed: 0, total: 0 };
    const cit = oracleResult.citations || { passed: 0, total: 0 };
    const totalPassed = (bib.passed || 0) + (cit.passed || 0);
    const totalItems = (bib.total || 0) + (cit.total || 0);
    const matchRate = totalItems > 0 ? totalPassed / totalItems : 0;
    return matchRate >= minPassRate ? 'pass' : 'fail';
  }
  return 'ok';
}

async function runBenchmarkRun(runtime, styleSpec, stylePath, styleYamlPath, benchmarkRun) {
  const resolvedRun = resolveBenchmarkRunConfig(benchmarkRun);
  try {
    if (resolvedRun.runner === BENCHMARK_RUNNERS.CITEPROC_ORACLE) {
      const oracleResult = await runCiteprocBenchmarkOracle(runtime, stylePath, styleSpec.name, resolvedRun);
      const benchmarkStatus = determineBenchmarkStatus(oracleResult, resolvedRun.minPassRate);
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
  const stylePath = stylePathOverride || path.join(path.dirname(__dirname), 'styles', 'embedded', `${styleName}.yaml`);
  if (!fs.existsSync(stylePath)) {
    const fallbackPath = path.join(path.dirname(__dirname), 'styles', `${styleName}.yaml`);
    if (fs.existsSync(fallbackPath)) {
      try {
        const rawStyleData = yaml.load(fs.readFileSync(fallbackPath, 'utf8'), { schema: CUSTOM_TAG_SCHEMA });
        return {
          stylePath: fallbackPath,
          rawStyleData,
          resolvedStyleData: resolveStyleData(rawStyleData),
          error: null,
        };
      } catch (error) {
        return {
          stylePath: fallbackPath,
          rawStyleData: null,
          resolvedStyleData: null,
          error: `YAML parse error: ${error.message}`,
        };
      }
    }
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
    if (Array.isArray(component.group)) {
      flattened.push(...flattenTemplateComponents(component.group));
    }
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
  const citationTypeVariants = citation['type-variants'] || {};
  const integralTypeVariants = citation.integral?.['type-variants'] || {};
  const nonIntegralTypeVariants = citation['non-integral']?.['type-variants'] || {};
  const bibliographyTypeVariants = bibliography['type-variants'] || {};
  const scopes = [];
  let variantSelectorCount = 0;

  function addScope(name, components, meta = {}) {
    if (!Array.isArray(components)) return;
    scopes.push({
      name,
      components,
      ...meta,
    });
  }

  function addVariantScopes(prefix, variants, kind) {
    for (const [rawKey, template] of Object.entries(variants || {})) {
      if (!Array.isArray(template)) continue;
      const typeSelectors = parseOverrideKey(rawKey).filter((key) => key !== 'default');
      variantSelectorCount += typeSelectors.length || 1;
      addScope(`${prefix}.${rawKey}`, template, {
        kind,
        typeSelectors,
        typeSelectorCount: typeSelectors.length || 1,
      });
    }
  }

  addScope('citation.template', citation.template, { kind: 'template' });
  addScope('citation.integral.template', citation.integral?.template, { kind: 'template' });
  addScope('citation.non-integral.template', citation['non-integral']?.template, { kind: 'template' });
  addScope('bibliography.template', bibliography.template, { kind: 'template' });
  addVariantScopes('citation.type-variants', citationTypeVariants, 'citation-type-variant');
  addVariantScopes('citation.integral.type-variants', integralTypeVariants, 'citation-type-variant');
  addVariantScopes('citation.non-integral.type-variants', nonIntegralTypeVariants, 'citation-type-variant');
  addVariantScopes('bibliography.type-variants', bibliographyTypeVariants, 'bibliography-type-variant');

  for (const [typeKey, template] of Object.entries(typeTemplates)) {
    if (!Array.isArray(template)) continue;
    const typeSelectors = parseOverrideKey(typeKey).filter((key) => key !== 'default');
    addScope(`bibliography.type-templates.${typeKey}`, template, {
      kind: 'bibliography-type-template',
      typeSelectors,
      typeSelectorCount: typeSelectors.length || 1,
    });
  }

  return { scopes, variantSelectorCount };
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

function stableStructure(value) {
  if (Array.isArray(value)) {
    return value.map(stableStructure);
  }
  if (!value || typeof value !== 'object') {
    return value;
  }
  const normalized = {};
  for (const key of Object.keys(value).sort()) {
    normalized[key] = stableStructure(value[key]);
  }
  return normalized;
}

function fingerprintValue(value) {
  return JSON.stringify(stableStructure(value));
}

function fingerprintComponent(component) {
  return fingerprintValue(component);
}

function fingerprintScopeComponents(components) {
  return fingerprintValue(components || []);
}

function collectPatternFingerprints(components, fingerprints = []) {
  for (const component of components || []) {
    if (!component || typeof component !== 'object') continue;
    const clone = { ...component };
    delete clone.overrides;
    fingerprints.push(fingerprintComponent(clone));
    if (Array.isArray(component.group)) {
      fingerprints.push(`group:${fingerprintValue(component.group)}`);
      collectPatternFingerprints(component.group, fingerprints);
    }
    if (Array.isArray(component.items)) {
      fingerprints.push(`items:${fingerprintValue(component.items)}`);
      collectPatternFingerprints(component.items, fingerprints);
    }
  }
  return fingerprints;
}

function componentDistance(left, right) {
  const leftShape = stableStructure(left);
  const rightShape = stableStructure(right);
  const leftKeys = new Set(Object.keys(leftShape || {}));
  const rightKeys = new Set(Object.keys(rightShape || {}));
  const allKeys = new Set([...leftKeys, ...rightKeys]);
  let distance = 0;

  for (const key of allKeys) {
    const leftValue = leftShape?.[key];
    const rightValue = rightShape?.[key];
    if (JSON.stringify(leftValue) !== JSON.stringify(rightValue)) {
      distance += 1;
    }
  }

  return distance;
}

function scopeDistance(leftComponents, rightComponents) {
  const left = leftComponents || [];
  const right = rightComponents || [];
  const maxLength = Math.max(left.length, right.length);
  let distance = Math.abs(left.length - right.length);

  for (let index = 0; index < maxLength; index += 1) {
    if (!left[index] || !right[index]) continue;
    distance += componentDistance(left[index], right[index]);
  }

  return distance;
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
  if (typeof bibliography['use-preset'] === 'string' && bibliography['use-preset'].trim()) {
    return {
      score: 100,
      assessedTypes: assessedTypes.length,
      passingTypes: assessedTypes.length,
      note: `embedded bibliography preset: ${bibliography['use-preset']} (assumed robust)`,
    };
  }
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

function computeConcisionScore(styleData, styleFormat) {
  const { scopes, variantSelectorCount } = collectTemplateScopes(styleData);
  const scopedComponents = scopes
    .map((scope) => ({
      name: scope.name,
      kind: scope.kind,
      typeSelectors: scope.typeSelectors || [],
      typeSelectorCount: scope.typeSelectorCount || 0,
      scopeFingerprint: fingerprintScopeComponents(scope.components),
      patternFingerprints: collectPatternFingerprints(scope.components),
      components: flattenTemplateComponents(scope.components),
      originalComponents: scope.components,
    }))
    .filter((scope) => scope.components.length > 0);
  const flattened = scopedComponents.flatMap((scope) => scope.components);

  if (flattened.length === 0) {
    return {
      score: 0,
      scopeCount: 0,
      totalComponents: 0,
      duplicates: 0,
      withinScopeDuplicates: 0,
      crossScopeRepeats: 0,
      exactDuplicateScopes: 0,
      nearDuplicateScopes: 0,
      repeatedPatterns: 0,
      variantSelectors: 0,
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

  const scopeFingerprintCounts = new Map();
  for (const scope of scopedComponents) {
    scopeFingerprintCounts.set(
      scope.scopeFingerprint,
      (scopeFingerprintCounts.get(scope.scopeFingerprint) || 0) + 1
    );
  }
  let exactDuplicateScopes = 0;
  for (const count of scopeFingerprintCounts.values()) {
    exactDuplicateScopes += Math.max(0, count - 1);
  }

  let nearDuplicateScopes = 0;
  const lengthBuckets = new Map();
  for (const scope of scopedComponents) {
    const len = scope.originalComponents.length;
    if (!lengthBuckets.has(len)) lengthBuckets.set(len, []);
    lengthBuckets.get(len).push(scope);
  }

  for (const bucket of lengthBuckets.values()) {
    for (let index = 0; index < bucket.length; index += 1) {
      for (let compareIndex = index + 1; compareIndex < bucket.length; compareIndex += 1) {
        const left = bucket[index];
        const right = bucket[compareIndex];
        if (left.scopeFingerprint === right.scopeFingerprint) continue;
        const distance = scopeDistance(left.originalComponents, right.originalComponents);
        if (distance > 0 && distance <= 2) {
          nearDuplicateScopes += 1;
        }
      }
    }
  }

  const patternCounts = new Map();
  for (const scope of scopedComponents) {
    const uniquePatterns = new Set(scope.patternFingerprints);
    for (const fingerprint of uniquePatterns) {
      patternCounts.set(fingerprint, (patternCounts.get(fingerprint) || 0) + 1);
    }
  }
  let repeatedPatterns = 0;
  for (const count of patternCounts.values()) {
    repeatedPatterns += Math.max(0, count - 1);
  }

  const weightedDuplicates = withinScopeDuplicates
    + (crossScopeRepeats * 0.35)
    + (exactDuplicateScopes * 2)
    + (nearDuplicateScopes * 1.25)
    + (repeatedPatterns * 0.15);
  const duplicateRatio = weightedDuplicates / Math.max(semanticKeys.length, 1);
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
  const targetBase = componentTargets[styleFormat] || 55;
  const targetComponents = targetBase + (variantSelectorCount * 1.5) + (typeTemplateCoverage * 1);
  const sizeRatio = flattened.length / Math.max(targetComponents, 1);

  // Score components:
  // 1. Duplicate penalty (weighted ratio)
  // 2. Size ratio (deviation from target complexity)
  // 3. Override density penalty
  const duplicateScore = clamp(0, 100, (1 - duplicateRatio) * 100);
  const sizeScore = clamp(0, 100, (1 / Math.max(sizeRatio, 1)) * 100);
  const overrideScore = clamp(0, 100, (1 - Math.min(overrideDensity, 0.5)) * 100);

  return {
    score: safePct((duplicateScore * 0.5) + (sizeScore * 0.35) + (overrideScore * 0.15)),
    scopeCount: scopedComponents.length,
    totalComponents: flattened.length,
    targetComponents: Math.round(targetComponents),
    duplicates: Math.round(weightedDuplicates),
    withinScopeDuplicates,
    crossScopeRepeats,
    exactDuplicateScopes,
    nearDuplicateScopes,
    repeatedPatterns,
    variantSelectors: variantSelectorCount,
    overrideDensity: parseFloat(overrideDensity.toFixed(2)),
  };
}

function computePresetUsageScore(styleData) {
  const templatePresets = countTemplatePresetUses(styleData);
  const optionsPresets = countOptionsPresetUses(styleData);
  const totalUses = templatePresets + optionsPresets.uses;

  if (totalUses >= 4) return { score: 100, uses: totalUses, fields: optionsPresets.fields };
  if (totalUses === 3) return { score: 85, uses: totalUses, fields: optionsPresets.fields };
  if (totalUses === 2) return { score: 70, uses: totalUses, fields: optionsPresets.fields };
  if (totalUses === 1) return { score: 50, uses: totalUses, fields: optionsPresets.fields };
  return { score: 0, uses: 0, fields: [] };
}

function computeQualityScore(qualityBreakdown) {
  const { subscores } = qualityBreakdown;
  const weights = {
    typeCoverage: 0.3,
    fallbackRobustness: 0.25,
    concision: 0.25,
    presetUsage: 0.2,
  };

  let totalWeight = 0;
  let weightedSum = 0;

  for (const [key, weight] of Object.entries(weights)) {
    if (subscores[key]) {
      weightedSum += subscores[key].score * weight;
      totalWeight += weight;
    }
  }

  return totalWeight > 0 ? safePct(weightedSum / totalWeight) : 0;
}

async function processStyleReport(runtime, styleSpec, verificationPolicy) {
  const styleYaml = loadStyleYaml(styleSpec.name);
  if (styleYaml.error) {
    return { name: styleSpec.name, error: styleYaml.error };
  }

  const stylePolicy = resolveVerificationPolicy(styleSpec.name, verificationPolicy);
  const sufficiencyPolicy = resolveFixtureSufficiency(
    stylePolicy.fixtureFamily || styleSpec.format,
    loadFixtureSufficiency()
  );

  const styleYamlPath = styleYaml.stylePath;
  const legacyStylePath = path.join(getStylesDir(runtime.stylesDir), `${styleSpec.sourceName}.csl`);
  const stylePath = fs.existsSync(legacyStylePath) ? legacyStylePath : styleYamlPath;

  let oracleResult;
  if (stylePolicy.authority === 'biblatex') {
    oracleResult = await runBiblatexSnapshotOracle(runtime, styleSpec.name, styleYamlPath, stylePolicy.authorityId);
  } else {
    oracleResult = await runCiteprocSnapshotOracle(runtime, stylePath, styleSpec.name, styleSpec.format);
  }

  const familySets = getAdditionalFixtureSetNames(sufficiencyPolicy.fixtureSets || []);
  const familyResults = await mapWithConcurrency(familySets, 2, (setName) =>
    runFamilyFixtureOracle(runtime, stylePath, styleSpec.name, setName)
  );

  for (const extra of familyResults) {
    mergeOracleResults(oracleResult, extra);
  }

  const benchmarkRunResults = await executeBenchmarkRuns(
    stylePolicy.benchmarkRuns,
    (run) => runBenchmarkRun(runtime, styleSpec, stylePath, styleYamlPath, run)
  );

  for (const benchmarkRunRecord of benchmarkRunResults) {
    mergeBenchmarkRunIntoOracle(oracleResult, benchmarkRunRecord);
  }

  const noteAudit = styleSpec.format === 'note'
    ? await auditNoteStyle(
        {
          name: styleSpec.name,
          path: styleYaml.stylePath,
          style: styleYaml.rawStyleData,
        },
        {
          citumBin: runtime.citumBin,
        }
      )
    : null;

  const qualityBreakdown = {
    subscores: {
      typeCoverage: computeTypeCoverageScore(oracleResult.citationsByType),
      fallbackRobustness: computeFallbackRobustness(styleYaml.resolvedStyleData),
      concision: computeConcisionScore(styleYaml.resolvedStyleData, styleSpec.format),
      presetUsage: computePresetUsageScore(styleYaml.rawStyleData),
    },
  };

  const fidelityScore = computeFidelityScore(oracleResult);
  const qualityScore = computeQualityScore(qualityBreakdown);

  return {
    ...styleSpec,
    fidelityScore,
    qualityScore,
    qualityBreakdown,
    caseMismatches: collectCaseMismatchSummary(oracleResult),
    rawCitations: summarizeSection(oracleResult.citations),
    rawBibliography: summarizeSection(oracleResult.bibliography),
    adjustedCitations: summarizeSection(oracleResult.adjusted.citations),
    adjustedBibliography: summarizeSection(oracleResult.adjusted.bibliography),
    divergenceSummary: oracleResult.adjusted.divergenceSummary,
    oracleSource: oracleResult.oracleSource,
    authorityLabel: formatAuthorityLabel(stylePolicy.authority, stylePolicy.authorityId),
    benchmarkRunResults: benchmarkRunResults.map(toPublishedBenchmarkRunRecord),
    notePositionAudit: noteAudit,
    error: oracleResult.error,
  };
}

async function generateReport(options = {}) {
  const runtime = createReportRuntime(options);
  const verificationPolicy = loadVerificationPolicy();
  const coreStyles = discoverCoreStyles();
  const selectedStyles = resolveSelectedStyles(coreStyles, options.styles || (options.styleName ? [options.styleName] : null));

  const results = await mapWithConcurrency(
    selectedStyles,
    options.parallelism || DEFAULT_PARALLELISM,
    (style) => processStyleReport(runtime, style, verificationPolicy)
  );

  const report = {
    generated: getTimestamp(),
    commit: getGitCommit(),
    totalStyles: results.length,
    totalImpact: results.reduce((sum, r) => sum + (r.dependents || 0), 0),
    citationsOverall: {
      passed: results.reduce((sum, r) => sum + (r.adjustedCitations?.passed || 0), 0),
      total: results.reduce((sum, r) => sum + (r.adjustedCitations?.total || 0), 0),
    },
    bibliographyOverall: {
      passed: results.reduce((sum, r) => sum + (r.adjustedBibliography?.passed || 0), 0),
      total: results.reduce((sum, r) => sum + (r.adjustedBibliography?.total || 0), 0),
    },
    qualityOverall: {
      score: safePct(results.reduce((sum, r) => sum + (r.qualityScore || 0), 0) / (results.length || 1)),
    },
    metadata: {
      styleSelector: options.styleName ? `style:${options.styleName}` : (options.styles ? 'selected-styles' : 'all-core'),
      styles: results.map((r) => r.name),
      richInputEvidence: options.styleName ? buildRichInputEvidenceSummary(results[0], results[0].benchmarkRunResults) : null,
    },
    styles: results,
  };

  if (options.timings) {
    report.timings = serializeTimingSummary(runtime);
  }

  return { report, runtime };
}

function getComparisonEntryTexts(entry) {
  if (entry.oracle && entry.citum) {
    return { benchmark: entry.oracle, citum: entry.citum };
  }
  if (entry.expected && entry.actual) {
    return { benchmark: entry.expected, citum: entry.actual };
  }
  return { benchmark: entry.oracle || entry.expected, citum: entry.citum || entry.actual };
}

function preflightSnapshots(selectedStyles, policy, stylesDir) {
  const issues = [];
  for (const style of selectedStyles) {
    const stylePolicy = resolveVerificationPolicy(style.name, policy);
    if (stylePolicy.authority === 'citeproc-js' || stylePolicy.secondary.includes('citeproc-js')) {
      const cslPath = path.join(stylesDir, `${style.sourceName}.csl`);
      if (!fs.existsSync(cslPath)) {
        issues.push({
          style: style.name,
          status: 'missing',
          message: `Legacy CSL not found: ${cslPath}`,
        });
      }
    }
  }
  return issues;
}

function generateHtml(report) {
  const templatePath = path.join(__dirname, 'report-data', 'compat-template.html');
  if (!fs.existsSync(templatePath)) {
    return JSON.stringify(report, null, 2);
  }

  const template = fs.readFileSync(templatePath, 'utf8');
  return template.replace('{{REPORT_JSON}}', JSON.stringify(report));
}

async function main() {
  const options = parseArgs();
  try {
    const { report } = await generateReport(options);

    if (options.writeHtml || options.outputHtml) {
      const html = generateHtml(report);
      const outputPath = options.outputHtml || path.join(PROJECT_ROOT, 'docs', 'compat.html');
      ensureDir(path.dirname(outputPath));
      fs.writeFileSync(outputPath, html);
      process.stderr.write(`Report written to ${outputPath}\n`);
    } else {
      process.stdout.write(JSON.stringify(report, null, 2));
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
  buildEmptyOracleResult,
  buildNoteStyleLookup,
  cloneOracleResult,
  collectTemplateScopes,
  computeConcisionScore,
  computeFidelityScore,
  computeImpactPct,
  determineBenchmarkStatus,
  discoverCoreStyles,
  equivalentText,
  executeBenchmarkRuns,
  expandCompoundBibEntries,
  formatAuthorityLabel,
  generateHtml,
  generateReport,
  getComparisonEntryTexts,
  getCslSnapshotStatus,
  getEffectiveOracleSection,
  hasBibliographyTemplate,
  inferStyleFormat,
  loadStyleYaml,
  mapWithConcurrency,
  mergeBenchmarkRunIntoOracle,
  mergeDivergenceSummaries,
  mergeOracleResults,
  parseArgs,
  preflightSnapshots,
  resolveSelectedStyles,
  runCachedJsonJob,
  selectPrimaryComparator,
  toPublishedBenchmarkRunRecord,
};
