#!/usr/bin/env node
/*
 * Converter SQI scorecard for citum-migrate.
 *
 * For each style in the chosen corpus this script:
 *   1. Invokes a prebuilt `citum-migrate` binary to produce migrated YAML.
 *   2. Reports fidelity via `scripts/oracle-migrate-batch.js` (force-migrate path).
 *   3. Scores the migrated YAML and the public YAML in `styles/` using the
 *      `concision`, `fallbackRobustness`, and `presetUsage` subscores exported
 *      from `scripts/report-core.js`.
 *   4. Aggregates corpus-level statistics and emits JSON + Markdown reports.
 *
 * The fourth SQI subscore, `typeCoverage`, depends on oracle per-type results
 * and is intentionally not part of the converter-output measurement: we want
 * to isolate the structural quality of the YAML the converter writes, not the
 * style's overall behavior. Fidelity is reported alongside for context but is
 * not folded into the SQI numbers.
 *
 * Usage:
 *   node scripts/report-migrate-sqi.js                       # default corpus (sentinels + lab)
 *   node scripts/report-migrate-sqi.js --corpus sentinels    # top-10 sentinels only
 *   node scripts/report-migrate-sqi.js --corpus lab          # migrate-research lab corpus only
 *   node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610
 *                                                            # seeded stratified random independents
 *   node scripts/report-migrate-sqi.js --styles apa,ieee     # explicit set
 *   node scripts/report-migrate-sqi.js --out /tmp/sqi.json   # write JSON to file
 *   node scripts/report-migrate-sqi.js --markdown docs/architecture/...md
 */

'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync, spawn } = require('child_process');

const reportCore = require('./report-core.js');
const { resolveAuthoredStylePath } = require('./oracle.js');
const { normalizeText } = require('./oracle-utils.js');
const { classifyCitationFormat, stratifiedSample } = require('./lib/corpus-sample.js');

const WORKSPACE_ROOT = path.resolve(__dirname, '..');
const LEGACY_DIR = path.join(WORKSPACE_ROOT, 'styles-legacy');
const STYLES_DIR = path.join(WORKSPACE_ROOT, 'styles');

// Resolved path to the citum-migrate binary, populated lazily on first use.
let MIGRATE_BIN = null;

// Cargo profile used to build the scorecard binary: optimized like release but
// `panic = "unwind"` (defined in the workspace Cargo.toml).
const MIGRATE_PROFILE = 'release-unwind';

/**
 * Resolve the citum-migrate binary, building it once under `release-unwind`.
 *
 * The binary MUST be built without `panic = "abort"`. The output-driven
 * synthesis loop scores many candidate templates and relies on
 * `std::panic::catch_unwind` to drop any candidate that panics in the engine
 * (see `catch_candidate_unwind` in `crates/citum-migrate/src/measured_citation.rs`).
 * Under the workspace `[profile.release] panic = "abort"`, `catch_unwind` is a
 * no-op, so a single panicking candidate aborts the whole migration (SIGABRT,
 * exit 134) and the style is reported as a spurious migrate failure. The
 * `release-unwind` profile keeps `panic = "unwind"`, so the loop's panic
 * isolation works — while still being optimized, so each style migrates in
 * ~1-2s instead of the ~16s a debug build takes.
 *
 * Building once and invoking the binary directly also removes the per-style
 * `cargo run` overhead (one cargo freshness check instead of 100). Set
 * `CITUM_MIGRATE_BIN` to a prebuilt unwinding binary to skip the build.
 */
function migrateBinary() {
  if (MIGRATE_BIN) return MIGRATE_BIN;
  const override = process.env.CITUM_MIGRATE_BIN;
  if (override) {
    if (!fs.existsSync(override)) {
      throw new Error(`CITUM_MIGRATE_BIN does not exist: ${override}`);
    }
    MIGRATE_BIN = override;
    return MIGRATE_BIN;
  }
  const exe = process.platform === 'win32' ? 'citum-migrate.exe' : 'citum-migrate';
  const profileBin = path.join(WORKSPACE_ROOT, 'target', MIGRATE_PROFILE, exe);
  process.stderr.write(`Building citum-migrate (${MIGRATE_PROFILE}, one-time freshness check)...\n`);
  const build = spawnSync(
    'cargo',
    ['build', '-q', '--profile', MIGRATE_PROFILE, '--bin', 'citum-migrate'],
    { cwd: WORKSPACE_ROOT, stdio: ['ignore', 'inherit', 'inherit'] }
  );
  if (build.status !== 0 || !fs.existsSync(profileBin)) {
    throw new Error('failed to build citum-migrate');
  }
  MIGRATE_BIN = profileBin;
  return MIGRATE_BIN;
}

/** Default worker-pool width: one per core, capped by the work item count. */
function defaultConcurrency(itemCount) {
  return Math.max(1, Math.min(itemCount, os.cpus().length));
}

/**
 * Run async `worker(item, index)` over `items` with at most `limit` in flight.
 *
 * Results preserve input order. The first rejection rejects the pool. Used to
 * fan the per-style migrate and the sharded oracle across cores, which is where
 * the scorecard spends almost all of its wall time.
 */
function runPool(items, limit, worker) {
  return new Promise((resolve, reject) => {
    const results = new Array(items.length);
    if (items.length === 0) {
      resolve(results);
      return;
    }
    let next = 0;
    let active = 0;
    let settled = 0;
    let failed = false;
    const pump = () => {
      while (!failed && active < limit && next < items.length) {
        const index = next;
        next += 1;
        active += 1;
        Promise.resolve(worker(items[index], index))
          .then((value) => {
            results[index] = value;
            active -= 1;
            settled += 1;
            if (settled === items.length) resolve(results);
            else pump();
          })
          .catch((err) => {
            if (!failed) {
              failed = true;
              reject(err);
            }
          });
      }
    };
    pump();
  });
}

/** Spawn a child process, buffering output; resolves with `{status, stdout, stderr}`. */
function spawnAsync(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: ['ignore', 'pipe', 'pipe'], ...opts });
    let stdout = '';
    let stderr = '';
    if (child.stdout) child.stdout.on('data', (chunk) => { stdout += chunk; });
    if (child.stderr) child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.on('error', reject);
    child.on('close', (status) => resolve({ status, stdout, stderr }));
  });
}

const MINIMIZATION_CITATIONS_FIXTURE = path.join(
  WORKSPACE_ROOT,
  'tests',
  'fixtures',
  'citations-minimization.json'
);

const SENTINELS = [
  'apa',
  'apa-6th-edition',
  'elsevier-harvard',
  'elsevier-with-titles',
  'elsevier-vancouver',
  'springer-basic-author-date',
  'ieee',
  'american-medical-association',
  'nature',
  'cell',
  'chicago-author-date',
  'chicago-notes',
  'oscola',
];

// Mirrors the migrate-research lab corpus (session 4 corpus minus styles
// already covered by sentinels).
const LAB_CORPUS = [
  'karger-journals',
  'institute-of-physics-numeric',
  'thieme-german',
  'multidisciplinary-digital-publishing-institute',
  'taylor-and-francis-chicago-author-date',
];

const DIAGNOSTIC_STYLES = [
  'apa-6th-edition',
];

const PATHOLOGICAL_OUTPUT_LINES = 1500;

// Headline quality bar: share of measured styles whose combined strict
// citation+bibliography pass rate reaches this fraction.
const FIDELITY_HEADLINE_THRESHOLD = 0.9;

// Default seed for `--corpus random`. Fixed so scheduled re-runs measure the
// same corpus and stay trend-comparable; pass `--seed` to draw a fresh one.
const DEFAULT_RANDOM_SEED = 20260610;
const DEFAULT_RANDOM_SAMPLE = 100;

function parseArgs(argv) {
  const args = {
    corpus: 'both',
    styles: null,
    out: null,
    markdown: null,
    json: false,
    skipFidelity: false,
    sample: DEFAULT_RANDOM_SAMPLE,
    seed: DEFAULT_RANDOM_SEED,
  };
  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--corpus' && argv[i + 1]) {
      args.corpus = argv[++i];
    } else if (arg === '--styles' && argv[i + 1]) {
      args.styles = argv[++i];
    } else if (arg === '--sample' && argv[i + 1]) {
      args.sample = Number(argv[++i]);
      if (!Number.isInteger(args.sample) || args.sample <= 0) {
        throw new Error('--sample must be a positive integer');
      }
    } else if (arg === '--seed' && argv[i + 1]) {
      args.seed = Number(argv[++i]);
      if (!Number.isInteger(args.seed)) {
        throw new Error('--seed must be an integer');
      }
    } else if (arg === '--out' && argv[i + 1]) {
      args.out = argv[++i];
    } else if (arg === '--markdown' && argv[i + 1]) {
      args.markdown = argv[++i];
    } else if (arg === '--json') {
      args.json = true;
    } else if (arg === '--skip-fidelity') {
      args.skipFidelity = true;
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
  console.log('Converter SQI scorecard for citum-migrate.');
  console.log('');
  console.log('Usage:');
  console.log('  node scripts/report-migrate-sqi.js');
  console.log('  node scripts/report-migrate-sqi.js --corpus sentinels|lab|both|random');
  console.log('  node scripts/report-migrate-sqi.js --corpus random --sample 100 --seed 20260610');
  console.log('  node scripts/report-migrate-sqi.js --styles apa,ieee');
  console.log('  node scripts/report-migrate-sqi.js --out /tmp/sqi.json --markdown docs/...md');
  console.log('  node scripts/report-migrate-sqi.js --skip-fidelity   # YAML scoring only, no oracle');
}

/** Top-level styles-legacy/*.csl names; dependents live under dependent/. */
function enumerateIndependentStyles() {
  return fs
    .readdirSync(LEGACY_DIR)
    .filter((name) => name.endsWith('.csl'))
    .map((name) => path.basename(name, '.csl'))
    .sort();
}

function classifyLegacyStyle(styleName) {
  try {
    const cslText = fs.readFileSync(path.join(LEGACY_DIR, `${styleName}.csl`), 'utf8');
    return classifyCitationFormat(cslText);
  } catch {
    return 'unknown';
  }
}

function resolveRandomCorpus(args) {
  const classified = enumerateIndependentStyles().map((style) => ({
    style,
    styleClass: classifyLegacyStyle(style),
  }));
  const drawn = stratifiedSample(classified, {
    sampleSize: args.sample,
    seed: args.seed,
  });
  return {
    styles: drawn.sample.map((entry) => entry.style),
    sampleMeta: {
      seed: args.seed,
      requested: args.sample,
      population: drawn.population,
      strata: drawn.strata,
      allocation: drawn.allocation,
    },
  };
}

function resolveCorpus(args) {
  if (args.styles) {
    return { styles: args.styles.split(',').map((s) => s.trim()).filter(Boolean) };
  }
  if (args.corpus === 'sentinels') return { styles: [...SENTINELS] };
  if (args.corpus === 'lab') return { styles: [...LAB_CORPUS] };
  if (args.corpus === 'both') return { styles: [...SENTINELS, ...LAB_CORPUS] };
  if (args.corpus === 'random') return resolveRandomCorpus(args);
  throw new Error(`Unknown corpus: ${args.corpus}`);
}

function inferStyleFormat(styleData) {
  // Mirror of report-core.js `inferStyleFormat` (not exported). Kept narrow:
  // we only need to distinguish 'note' from anything else for concision scoring.
  const processing = styleData?.options?.processing;
  if (typeof processing === 'string') return processing;
  if (processing && typeof processing === 'object') {
    if (Object.prototype.hasOwnProperty.call(processing, 'note')) return 'note';
    if (Object.prototype.hasOwnProperty.call(processing, 'author-date')) return 'author-date';
    if (Object.prototype.hasOwnProperty.call(processing, 'numeric')) return 'numeric';
  }
  return 'author-date';
}

function migrateStyleToYaml(styleName) {
  const cslPath = path.join(LEGACY_DIR, `${styleName}.csl`);
  if (!fs.existsSync(cslPath)) {
    return { error: 'missing_legacy_style', details: cslPath };
  }
  const evidenceDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-evidence-'));
  const evidenceTmp = path.join(evidenceDir, `${styleName}.evidence.json`);
  try {
    const proc = spawnSync(
      migrateBinary(),
      [
        '--emit-evidence', evidenceTmp,
        cslPath,
      ],
      { cwd: WORKSPACE_ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], maxBuffer: 64 * 1024 * 1024 }
    );
    if (proc.status !== 0) {
      return {
        error: 'migrate_failed',
        details: (proc.stderr || '').trim() || `exit=${proc.status}`,
      };
    }
    let evidence = null;
    try {
      if (fs.existsSync(evidenceTmp)) {
        evidence = JSON.parse(fs.readFileSync(evidenceTmp, 'utf8'));
      }
    } catch (err) {
      // Non-fatal: scorecard still scores YAML even without an evidence sidecar.
      evidence = { error: 'evidence_parse_failed', details: err.message };
    }
    return { yaml: proc.stdout, evidence };
  } finally {
    // Always clean up the evidence tmp dir, even on early-return error paths.
    fs.rmSync(evidenceDir, { recursive: true, force: true });
  }
}

/**
 * Async counterpart of {@link migrateStyleToYaml}: spawns the migrate binary
 * without blocking the event loop so {@link migrateAllParallel} can fan many
 * migrations across cores. Same return shape (`{yaml, evidence}` or `{error}`).
 */
async function migrateStyleAsync(styleName) {
  const cslPath = path.join(LEGACY_DIR, `${styleName}.csl`);
  if (!fs.existsSync(cslPath)) {
    return { error: 'missing_legacy_style', details: cslPath };
  }
  const evidenceDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-evidence-'));
  const evidenceTmp = path.join(evidenceDir, `${styleName}.evidence.json`);
  try {
    const proc = await spawnAsync(
      migrateBinary(),
      ['--emit-evidence', evidenceTmp, cslPath],
      { cwd: WORKSPACE_ROOT }
    );
    if (proc.status !== 0) {
      return {
        error: 'migrate_failed',
        details: (proc.stderr || '').trim() || `exit=${proc.status}`,
      };
    }
    let evidence = null;
    try {
      if (fs.existsSync(evidenceTmp)) {
        evidence = JSON.parse(fs.readFileSync(evidenceTmp, 'utf8'));
      }
    } catch (err) {
      evidence = { error: 'evidence_parse_failed', details: err.message };
    }
    return { yaml: proc.stdout, evidence };
  } finally {
    fs.rmSync(evidenceDir, { recursive: true, force: true });
  }
}

/**
 * Migrate every style concurrently, returning a `Map<style, result>` keyed by
 * style name. Builds the migrate binary once (via the first `migrateBinary()`
 * call) before fanning out, so the workers share one prebuilt binary.
 */
async function migrateAllParallel(styles) {
  migrateBinary();
  const map = new Map();
  await runPool(styles, defaultConcurrency(styles.length), async (style) => {
    map.set(style, await migrateStyleAsync(style));
  });
  return map;
}

/**
 * Attempt minimization for a style with a discovered family-candidate parent.
 * Runs `citum-migrate --family-candidate auto --minimize-wrapper` and returns
 * the minimized YAML + evidence. Caller is responsible for oracle-verifying
 * the result before swapping it in. Returns null when the style has no
 * candidate (no minimization possible) or migration fails.
 */
function attemptMinimization(styleName) {
  const cslPath = path.join(LEGACY_DIR, `${styleName}.csl`);
  if (!fs.existsSync(cslPath)) return null;
  const evidenceDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-minev-'));
  const evidenceTmp = path.join(evidenceDir, `${styleName}.evidence.json`);
  try {
    const proc = spawnSync(
      migrateBinary(),
      [
        '--family-candidate', 'auto',
        '--minimize-wrapper',
        '--emit-evidence', evidenceTmp,
        cslPath,
      ],
      { cwd: WORKSPACE_ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], maxBuffer: 64 * 1024 * 1024 }
    );
    if (proc.status !== 0) return null;
    let evidence = null;
    try {
      if (fs.existsSync(evidenceTmp)) {
        evidence = JSON.parse(fs.readFileSync(evidenceTmp, 'utf8'));
      }
    } catch {
      evidence = null;
    }
    return { yaml: proc.stdout, evidence };
  } finally {
    fs.rmSync(evidenceDir, { recursive: true, force: true });
  }
}

/**
 * Run the migrate-batch oracle on a pre-built YAML by writing it to a temp
 * file named after the style and invoking `oracle.js` with that path.
 * Returns the parsed oracle section objects for `citations` and
 * `bibliography`, including their `entries` arrays when present. Either
 * section can be `null` if the oracle did not produce that section. Returns
 * `null` when the oracle output cannot be parsed or lacks both sections.
 */
function oracleForYaml(styleName, yamlText, options = {}) {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-min-oracle-'));
  const yamlPath = path.join(tmpDir, `${styleName}.yaml`);
  try {
    fs.writeFileSync(yamlPath, yamlText);
    const oracleArgs = [path.join(WORKSPACE_ROOT, 'scripts', 'oracle.js'), yamlPath, '--json'];
    if (options.citationsFixture) {
      oracleArgs.push('--citations-fixture', options.citationsFixture);
    }
    const proc = spawnSync(
      'node',
      oracleArgs,
      { cwd: WORKSPACE_ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], maxBuffer: 64 * 1024 * 1024 }
    );
    // oracle.js exits with status 1 whenever fidelity is below 100%, even
    // when stdout contains a well-formed JSON report. Treat the run as
    // successful as long as the stdout parses and carries the required
    // citation/bibliography aggregates; surface failure only when the JSON
    // itself is malformed or missing.
    if (!proc.stdout) return null;
    const parsed = JSON.parse(proc.stdout);
    if (!parsed.citations && !parsed.bibliography) return null;
    return {
      citations: parsed.citations || null,
      bibliography: parsed.bibliography || null,
    };
  } catch {
    return null;
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

function sectionPassCount(section) {
  return section?.passed ?? 0;
}

function normalizedEqual(left, right) {
  if (typeof left !== 'string' || typeof right !== 'string') {
    return false;
  }
  // Minimization-acceptance checks compare oracle snapshots against each other
  // (not citum's rendering against the oracle), so a stray trailing-punctuation
  // difference here is noise, not the class of defect this comparison guards
  // against -- unlike the shared normalizeText used by the fidelity harness,
  // this strip stays local to this file.
  const stripTrailingPunctuation = (text) => text.replace(/[.,;:]\s*$/, '');
  return stripTrailingPunctuation(normalizeText(left)) === stripTrailingPunctuation(normalizeText(right));
}

function strictSectionEquivalent(section) {
  if (!section || !Array.isArray(section.entries) || section.entries.length === 0) {
    return false;
  }
  return section.entries.every((entry) => normalizedEqual(entry.oracle, entry.citum));
}

function evaluateMinimizationAcceptance({ baseOracle, minOracle, minLoc, baseLoc }) {
  const strict = {
    citations: strictSectionEquivalent(minOracle?.citations),
    bibliography: strictSectionEquivalent(minOracle?.bibliography),
  };
  const passCountsHold = minOracle != null
    && sectionPassCount(minOracle.citations) >= sectionPassCount(baseOracle?.citations)
    && sectionPassCount(minOracle.bibliography) >= sectionPassCount(baseOracle?.bibliography);
  const locImproves = minLoc < baseLoc;
  return {
    accepted: passCountsHold && locImproves && strict.citations && strict.bibliography,
    strict,
    passCountsHold,
    locImproves,
  };
}

function stripCustomYamlTags(yamlText) {
  // citum-migrate emits serde-tagged enum variants like `processing: !custom`.
  // js-yaml (used by report-core) rejects unknown tags. The mapping body that
  // follows is what we want to score, so drop the inline tag verb only.
  return yamlText.replace(/(:\s)![A-Za-z][A-Za-z0-9_-]*(\s|$)/g, '$1$2');
}

function loadStyleFromYamlText(styleName, yamlText) {
  // Write to a temp file and reuse report-core's loader so resolution behaves
  // identically to the in-repo path (extends, presets, etc.).
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-sqi-'));
  const tmpPath = path.join(tmpDir, `${styleName}.yaml`);
  fs.writeFileSync(tmpPath, stripCustomYamlTags(yamlText));
  const loaded = reportCore.loadStyleYaml(styleName, tmpPath);
  return { loaded, cleanup: () => fs.rmSync(tmpDir, { recursive: true, force: true }) };
}

function loadPublicStyle(styleName) {
  // Delegate to oracle.js so the embedded/ prefix-scan rules stay in one place.
  const stylePath = resolveAuthoredStylePath(STYLES_DIR, styleName);
  if (!stylePath) {
    return { error: `Public style YAML not found for ${styleName}` };
  }
  return reportCore.loadStyleYaml(styleName, stylePath);
}

function scoreYaml(loaded, yamlText = null) {
  if (!loaded || !loaded.resolvedStyleData) {
    return { error: loaded?.error || 'no_yaml' };
  }
  const data = loaded.rawStyleData || loaded.resolvedStyleData;
  const format = inferStyleFormat(data);
  const concision = reportCore.computeConcisionScore(data, format);
  const fallback = reportCore.computeFallbackRobustness(loaded.resolvedStyleData);
  const presetUsage = reportCore.computePresetUsageScore(data, concision.score);
  const composite = (concision.score + fallback.score + presetUsage.score) / 3;
  return {
    composite: Number(composite.toFixed(2)),
    concision: concision.score,
    fallbackRobustness: fallback.score,
    presetUsage: presetUsage.score,
    diagnostics: {
      scopeCount: concision.scopeCount,
      variantSelectors: concision.variantSelectors,
      exactDuplicateScopes: concision.exactDuplicateScopes,
      nearDuplicateScopes: concision.nearDuplicateScopes,
      repeatedPatterns: concision.repeatedPatterns,
      diffVariantScopes: concision.diffVariantScopes,
      diffVariantOperations: concision.diffVariantOperations,
      inheritedPreset: concision.inheritedPreset || null,
      hasRootExtends: Boolean(data.extends),
      outputLines: yamlText ? countLines(yamlText) : null,
      templateComponents: countTemplateComponents(data),
      bibliographyTemplateComponents: countSectionTemplateComponents(data.bibliography),
      bibliographyTypeVariantScopes: countTypeVariantScopes(data.bibliography),
      citationTemplateComponents: countSectionTemplateComponents(data.citation),
      citationTypeVariantScopes: countTypeVariantScopes(data.citation),
      pathologicalOutput: yamlText ? countLines(yamlText) > PATHOLOGICAL_OUTPUT_LINES : false,
    },
  };
}

function countLines(text) {
  return text.trimEnd().split(/\r?\n/).length;
}

function countTemplateComponents(styleData) {
  return countSectionTemplateComponents(styleData?.citation)
    + countSectionTemplateComponents(styleData?.bibliography)
    + countSectionTypeVariantComponents(styleData?.citation)
    + countSectionTypeVariantComponents(styleData?.bibliography);
}

function countSectionTemplateComponents(section) {
  return countComponentList(section?.template);
}

function countSectionTypeVariantComponents(section) {
  const variants = section?.['type-variants'];
  if (!variants || typeof variants !== 'object') return 0;
  let total = 0;
  for (const variant of Object.values(variants)) {
    if (Array.isArray(variant)) {
      total += countComponentList(variant);
    } else if (variant && typeof variant === 'object') {
      total += countComponentList(variant.add?.map((op) => op.component));
    }
  }
  return total;
}

function countTypeVariantScopes(section) {
  const variants = section?.['type-variants'];
  if (!variants || typeof variants !== 'object') return 0;
  return Object.keys(variants).length;
}

function countComponentList(list) {
  if (!Array.isArray(list)) return 0;
  let total = 0;
  for (const component of list) {
    total += countComponent(component);
  }
  return total;
}

function countComponent(component) {
  if (!component || typeof component !== 'object') return 0;
  if (Array.isArray(component.group)) {
    return 1 + countComponentList(component.group);
  }
  return 1;
}

/**
 * Render citeproc-js fidelity for one shard of styles via
 * `oracle-migrate-batch.js`, returning the parsed per-style rows. Each shard
 * runs in its own subprocess with a private `--out` file, so shards are safe
 * to run concurrently.
 */
async function runOracleShard(styles) {
  // Read results via --out instead of stdout: a large corpus (100+ styles)
  // overflows the child's stdout buffer and kills it with a null exit status.
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-oracle-batch-'));
  const outPath = path.join(tmpDir, 'oracle-batch.json');
  try {
    const oracleArgs = [
      path.join(WORKSPACE_ROOT, 'scripts', 'oracle-migrate-batch.js'),
      '--styles', styles.join(','),
      '--out', outPath,
    ];
    const proc = await spawnAsync('node', oracleArgs, { cwd: WORKSPACE_ROOT });
    if (proc.status !== 0 || !fs.existsSync(outPath)) {
      const detail = (proc.stderr || '').trim() || `exit=${proc.status}`;
      throw new Error(`oracle-migrate-batch failed: ${detail}`);
    }
    const summary = JSON.parse(fs.readFileSync(outPath, 'utf8'));
    return summary.styles || [];
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

/**
 * Render citeproc-js fidelity for all styles, sharding across cores. The
 * citeproc oracle dominates scorecard wall time, so splitting the corpus into
 * one shard per core and running them concurrently is the primary speedup.
 * Returns a `Map<style, row>`.
 */
async function runOracle(styles) {
  const concurrency = defaultConcurrency(styles.length);
  const shards = Array.from({ length: concurrency }, () => []);
  styles.forEach((style, index) => shards[index % concurrency].push(style));
  const populated = shards.filter((shard) => shard.length > 0);
  const shardRows = await runPool(populated, concurrency, (shard) => runOracleShard(shard));
  const fidelity = new Map();
  for (const rows of shardRows) {
    for (const row of rows) fidelity.set(row.style, row);
  }
  return fidelity;
}

function percentile(sorted, p) {
  if (sorted.length === 0) return null;
  const idx = Math.min(sorted.length - 1, Math.max(0, Math.floor(sorted.length * p)));
  return Number(sorted[idx].toFixed(2));
}

function aggregateComposite(rows, key) {
  const values = rows
    .map((row) => row[key]?.composite)
    .filter((value) => typeof value === 'number')
    .sort((a, b) => a - b);
  if (values.length === 0) return null;
  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
  return {
    n: values.length,
    mean: Number(mean.toFixed(2)),
    p10: percentile(values, 0.1),
    p50: percentile(values, 0.5),
    p90: percentile(values, 0.9),
  };
}

function aggregateDelta(rows) {
  const deltas = rows
    .filter((row) => row.migrated?.composite != null && row.public?.composite != null)
    .map((row) => row.migrated.composite - row.public.composite)
    .sort((a, b) => a - b);
  if (deltas.length === 0) return null;
  const mean = deltas.reduce((sum, value) => sum + value, 0) / deltas.length;
  return {
    n: deltas.length,
    mean: Number(mean.toFixed(2)),
    p10: percentile(deltas, 0.1),
    p50: percentile(deltas, 0.5),
    p90: percentile(deltas, 0.9),
  };
}

/**
 * Per-style measurement status for the fidelity headline. Conversion or
 * oracle failures are data, not script errors: they count against the
 * headline rather than aborting the run.
 */
function rowFidelityStatus(row) {
  if (row.error) return 'migrate_failed';
  if (row.fidelity?.error) return 'oracle_failed';
  if (!row.fidelity) return 'fidelity_skipped';
  const total = (row.fidelity.citations?.total || 0) + (row.fidelity.bibliography?.total || 0);
  if (total === 0) return 'oracle_empty';
  return 'ok';
}

/** Combined strict pass rate across citations + bibliography, or null. */
function combinedFidelity(row) {
  if (rowFidelityStatus(row) !== 'ok') return null;
  const passed = (row.fidelity.citations?.passed || 0) + (row.fidelity.bibliography?.passed || 0);
  const total = (row.fidelity.citations?.total || 0) + (row.fidelity.bibliography?.total || 0);
  return passed / total;
}

function summarizeFidelity(rows, threshold) {
  const statuses = {};
  const values = [];
  let atThreshold = 0;
  for (const row of rows) {
    const status = rowFidelityStatus(row);
    statuses[status] = (statuses[status] || 0) + 1;
    const fidelity = combinedFidelity(row);
    if (fidelity != null) {
      values.push(fidelity);
      if (fidelity >= threshold) atThreshold += 1;
    }
  }
  // Failures stay in the denominator: a style that cannot convert does not
  // meet the bar, and hiding it would inflate the published number.
  const measured = rows.length - (statuses.fidelity_skipped || 0);
  values.sort((a, b) => a - b);
  const pct = (value) => Number((value * 100).toFixed(1));
  const mean = values.length
    ? values.reduce((sum, value) => sum + value, 0) / values.length
    : null;
  return {
    measured,
    statuses,
    atThreshold,
    shareAtThreshold: measured ? pct(atThreshold / measured) : null,
    combined: values.length
      ? {
          n: values.length,
          mean: pct(mean),
          p10: pct(values[Math.min(values.length - 1, Math.floor(values.length * 0.1))]),
          p50: pct(values[Math.min(values.length - 1, Math.floor(values.length * 0.5))]),
          p90: pct(values[Math.min(values.length - 1, Math.floor(values.length * 0.9))]),
        }
      : null,
  };
}

/**
 * Headline fidelity aggregates over scorecard rows: overall and per
 * style-class summaries against FIDELITY_HEADLINE_THRESHOLD.
 */
function aggregateFidelityHeadline(rows, threshold = FIDELITY_HEADLINE_THRESHOLD) {
  const measurable = rows.filter((row) => rowFidelityStatus(row) !== 'fidelity_skipped');
  if (measurable.length === 0) return null;
  const byClass = new Map();
  for (const row of rows) {
    const styleClass = row.styleClass || 'unknown';
    if (!byClass.has(styleClass)) byClass.set(styleClass, []);
    byClass.get(styleClass).push(row);
  }
  const perClass = {};
  for (const [styleClass, classRows] of [...byClass.entries()].sort()) {
    perClass[styleClass] = summarizeFidelity(classRows, threshold);
  }
  return {
    threshold: Number((threshold * 100).toFixed(1)),
    overall: summarizeFidelity(rows, threshold),
    perClass,
  };
}

function buildMarkdown(report) {
  const lines = [];
  lines.push(`# citum-migrate SQI baseline`);
  lines.push('');
  lines.push(`- Generated: ${report.generated}`);
  lines.push(`- Commit: ${report.commit}`);
  if (report.sample) {
    lines.push(
      `- Corpus: random (${report.results.length} of ${report.sample.population} independent parents, seed ${report.sample.seed})`
    );
    const strata = Object.entries(report.sample.allocation)
      .map(([name, count]) => `${name} ${count}/${report.sample.strata[name]}`)
      .join(', ');
    lines.push(`- Strata (sampled/population): ${strata}`);
  } else {
    lines.push(`- Corpus: ${report.corpus} (${report.results.length} styles)`);
  }
  const headline = report.aggregate.fidelityHeadline;
  if (headline) {
    lines.push('');
    lines.push('## Fidelity headline');
    lines.push('');
    const overall = headline.overall;
    const statuses = Object.entries(overall.statuses)
      .filter(([status]) => status !== 'ok')
      .map(([status, count]) => `${status} ${count}`)
      .join(', ');
    lines.push(
      `- Styles at ≥${headline.threshold}% combined strict fidelity: ${overall.atThreshold}/${overall.measured} (${overall.shareAtThreshold}%)${statuses ? ` — non-ok: ${statuses}` : ''}`
    );
    if (overall.combined) {
      lines.push(
        `- Combined strict fidelity (converted styles): mean ${overall.combined.mean}%, p10 ${overall.combined.p10}%, median ${overall.combined.p50}%, p90 ${overall.combined.p90}%`
      );
    }
    lines.push('');
    lines.push('### Per class');
    lines.push('');
    lines.push(`| Class | Measured | ≥${headline.threshold}% | Share | Mean | Median |`);
    lines.push('|---|---:|---:|---:|---:|---:|');
    for (const [styleClass, summary] of Object.entries(headline.perClass)) {
      lines.push(
        `| ${styleClass} | ${summary.measured} | ${summary.atThreshold} | ${summary.shareAtThreshold ?? '-'}% | ${summary.combined?.mean ?? '-'}% | ${summary.combined?.p50 ?? '-'}% |`
      );
    }
    const failures = report.results.filter((row) =>
      ['migrate_failed', 'oracle_failed', 'oracle_empty'].includes(rowFidelityStatus(row))
    );
    if (failures.length > 0) {
      lines.push('');
      lines.push('### Failure taxonomy');
      lines.push('');
      lines.push('| Style | Class | Status | Detail |');
      lines.push('|---|---|---|---|');
      for (const row of failures) {
        const status = rowFidelityStatus(row);
        const detail = String(
          row.error?.details || row.fidelity?.details || row.fidelity?.error || ''
        )
          .replace(/\s+/g, ' ')
          .slice(0, 140);
        lines.push(`| ${row.style} | ${row.styleClass ?? '-'} | ${status} | ${detail} |`);
      }
    }
  }
  lines.push('');
  lines.push('## Aggregate');
  lines.push('');
  lines.push('| Subject | n | mean | p10 | p50 | p90 |');
  lines.push('|---|---:|---:|---:|---:|---:|');
  const m = report.aggregate.migrated;
  const p = report.aggregate.public;
  const d = report.aggregate.delta;
  if (m) lines.push(`| Migrated YAML SQI | ${m.n} | ${m.mean} | ${m.p10} | ${m.p50} | ${m.p90} |`);
  if (p) lines.push(`| Public YAML SQI | ${p.n} | ${p.mean} | ${p.p10} | ${p.p50} | ${p.p90} |`);
  if (d) lines.push(`| Migrated − Public | ${d.n} | ${d.mean} | ${d.p10} | ${d.p50} | ${d.p90} |`);
  lines.push('');
  lines.push('## Per-style');
  lines.push('');
  lines.push('| Style | Fidelity | Migrated SQI | Public SQI | Δ | LOC | Migrated dup/near/rep | Public dup/near/rep |');
  lines.push('|---|---:|---:|---:|---:|---:|---|---|');
  for (const row of report.results) {
    const fid = row.fidelity
      ? `${row.fidelity.citations?.passed ?? '-'}/${row.fidelity.citations?.total ?? '-'} • ${row.fidelity.bibliography?.passed ?? '-'}/${row.fidelity.bibliography?.total ?? '-'}`
      : 'skipped';
    const mig = row.migrated?.composite ?? 'err';
    const pub = row.public?.composite ?? 'err';
    const delta = (typeof mig === 'number' && typeof pub === 'number')
      ? (mig - pub).toFixed(2)
      : '-';
    const migDiag = row.migrated?.diagnostics
      ? `${row.migrated.diagnostics.exactDuplicateScopes}/${row.migrated.diagnostics.nearDuplicateScopes}/${row.migrated.diagnostics.repeatedPatterns}`
      : '-';
    const pubDiag = row.public?.diagnostics
      ? `${row.public.diagnostics.exactDuplicateScopes}/${row.public.diagnostics.nearDuplicateScopes}/${row.public.diagnostics.repeatedPatterns}`
      : '-';
    const loc = row.migrated?.diagnostics?.outputLines ?? '-';
    lines.push(`| ${row.style} | ${fid} | ${mig} | ${pub} | ${delta} | ${loc} | ${migDiag} | ${pubDiag} |`);
  }
  lines.push('');
  lines.push('Columns: Migrated/Public SQI is a simple mean of `concision`, `fallbackRobustness`, and `presetUsage` (0–100). LOC is migrated YAML output lines. dup/near/rep counts come from `qualityBreakdown.subscores.concision` diagnostics in `report-core.js`.');
  const candidates = report.results.filter((row) =>
    Array.isArray(row.evidence?.discovered_parents) && row.evidence.discovered_parents.length > 0
  );
  if (candidates.length > 0) {
    lines.push('');
    lines.push('## Compression candidates');
    lines.push('');
    lines.push('Styles where the migrator discovered a candidate parent via the registry, a source CSL link, or a reverse `<info><link rel="template">` in an embedded canonical style. The scorecard tries the minimized wrapper form (`--family-candidate auto --minimize-wrapper`) for each candidate and accepts it only when oracle citation pass ≥ standalone, bibliography pass ≥ standalone, LOC decreases, and every minimized citation/bibliography entry is strictly equivalent after normalization.');
    lines.push('');
    lines.push('| Style | Candidate parent | Discovery source | Standalone LOC → Minimized LOC | Standalone fidelity → Minimized fidelity | Accepted |');
    lines.push('|---|---|---|---:|---|:---:|');
    for (const row of candidates) {
      const candidate = row.evidence.discovered_parents[0];
      const m = row.minimization;
      const standaloneLoc = m?.standalone?.outputLines ?? row.evidence.standalone_output_lines ?? '-';
      const minimizedLoc = m?.minimized?.outputLines ?? '-';
      const fidShort = (fid) => fid
        ? `${fid.citations?.passed ?? '-'}/${fid.citations?.total ?? '-'} • ${fid.bibliography?.passed ?? '-'}/${fid.bibliography?.total ?? '-'}`
        : '-';
      const standaloneFid = fidShort(m?.standalone);
      const minimizedFid = fidShort(m?.minimized);
      const accepted = m?.accepted === true ? '✓' : (m?.attempted ? '✗' : '–');
      lines.push(`| ${row.style} | ${candidate.canonical_id} | ${candidate.source} | ${standaloneLoc} → ${minimizedLoc} | ${standaloneFid} → ${minimizedFid} | ${accepted} |`);
    }
  }
  if (report.diagnostics?.migratedOutputs?.length) {
    lines.push('');
    lines.push('## Output Diagnostics');
    lines.push('');
    lines.push('| Style | LOC | Template components | Bibliography template | Bibliography variants | Pathological |');
    lines.push('|---|---:|---:|---:|---:|---|');
    for (const row of report.diagnostics.migratedOutputs) {
      const diag = row.migrated?.diagnostics;
      lines.push(`| ${row.style} | ${diag?.outputLines ?? '-'} | ${diag?.templateComponents ?? '-'} | ${diag?.bibliographyTemplateComponents ?? '-'} | ${diag?.bibliographyTypeVariantScopes ?? '-'} | ${diag?.pathologicalOutput ? 'yes' : 'no'} |`);
    }
  }
  lines.push('');
  return lines.join('\n');
}

function gitCommit() {
  const proc = spawnSync('git', ['rev-parse', '--short', 'HEAD'], {
    cwd: WORKSPACE_ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (proc.status !== 0) return 'unknown';
  return proc.stdout.trim();
}

async function main() {
  const args = parseArgs(process.argv);
  const { styles, sampleMeta } = resolveCorpus(args);
  // Precompute the two dominant costs concurrently: the per-style migrations
  // (fanned across cores) and the sharded citeproc oracle. The per-style loop
  // below then reads from these caches instead of spawning serially.
  const [migratedByStyle, fidelity] = await Promise.all([
    migrateAllParallel(styles),
    args.skipFidelity ? Promise.resolve(new Map()) : runOracle(styles),
  ]);

  const results = [];
  for (const style of styles) {
    const row = { style, styleClass: classifyLegacyStyle(style) };
    const migrated = migratedByStyle.get(style) || migrateStyleToYaml(style);
    if (migrated.error) {
      row.error = migrated;
    } else {
      const { loaded, cleanup } = loadStyleFromYamlText(style, migrated.yaml);
      try {
        row.migrated = scoreYaml(loaded, migrated.yaml);
      } finally {
        cleanup();
      }
      if (migrated.evidence) {
        row.evidence = migrated.evidence;
      }
    }
    const publicLoaded = loadPublicStyle(style);
    row.public = scoreYaml(publicLoaded);
    const fidelityRow = fidelity.get(style);
    if (fidelityRow) {
      row.fidelity = {
        citations: fidelityRow.citations,
        bibliography: fidelityRow.bibliography,
        error: fidelityRow.error || null,
        details: fidelityRow.details || null,
      };
    }
    // Attempt evidence-driven wrapper minimization only for styles the
    // converter currently emits as standalone with a discovered candidate
    // parent. Styles already routed through `ExistingWrapper` at lineage
    // time (registry aliases, descendant wrappers) need no further
    // compression and would otherwise show as no-op minimization.
    const isStandaloneEmission = row.evidence?.emitted_form === 'standalone';
    if (
      !args.skipFidelity
      && !migrated.error
      && isStandaloneEmission
      && Array.isArray(row.evidence?.discovered_parents)
      && row.evidence.discovered_parents.length > 0
    ) {
      const min = attemptMinimization(style);
      if (min && min.yaml) {
        const strictFixtureOptions = { citationsFixture: MINIMIZATION_CITATIONS_FIXTURE };
        const baseStrictOracle = oracleForYaml(style, migrated.yaml, strictFixtureOptions);
        const minOracle = oracleForYaml(style, min.yaml, strictFixtureOptions);
        const minLoc = min.evidence?.emitted_output_lines ?? Number.MAX_SAFE_INTEGER;
        const baseLoc = row.evidence?.standalone_output_lines
          ?? row.migrated?.diagnostics?.outputLines
          ?? Number.MAX_SAFE_INTEGER;
        // Acceptance requires: fidelity holds (citations and bibliography
        // pass counts do not regress), and the minimized form is actually
        // smaller than standalone. Equal-size results indicate the
        // converter did not promote the family candidate (e.g. mdpi's
        // template-link parent path bypasses the minimize branch); marking
        // those as compressed would be misleading.
        const acceptance = evaluateMinimizationAcceptance({
          baseOracle: baseStrictOracle ?? row.fidelity,
          minOracle,
          minLoc,
          baseLoc,
        });
        const accepted = acceptance.accepted;
        row.minimization = {
          attempted: true,
          accepted,
          strict: acceptance.strict,
          passCountsHold: acceptance.passCountsHold,
          locImproves: acceptance.locImproves,
          standalone: {
            outputLines: row.migrated?.diagnostics?.outputLines ?? null,
            citations: baseStrictOracle?.citations ?? row.fidelity?.citations ?? null,
            bibliography: baseStrictOracle?.bibliography ?? row.fidelity?.bibliography ?? null,
          },
          minimized: {
            outputLines: min.evidence?.emitted_output_lines ?? null,
            citations: minOracle?.citations ?? null,
            bibliography: minOracle?.bibliography ?? null,
          },
        };
        if (accepted) {
          // Swap row's reported migrated form to the minimized YAML so SQI
          // and LOC reflect what the converter can actually emit.
          const { loaded, cleanup } = loadStyleFromYamlText(style, min.yaml);
          try {
            row.migrated = scoreYaml(loaded, min.yaml);
          } finally {
            cleanup();
          }
          row.evidence = min.evidence;
          row.fidelity = {
            citations: minOracle.citations,
            bibliography: minOracle.bibliography,
            error: null,
          };
        }
      }
    }
    results.push(row);
  }

  const report = {
    generated: new Date().toISOString(),
    commit: gitCommit(),
    corpus: args.styles ? 'explicit' : args.corpus,
    ...(sampleMeta ? { sample: sampleMeta } : {}),
    aggregate: {
      migrated: aggregateComposite(results, 'migrated'),
      public: aggregateComposite(results, 'public'),
      delta: aggregateDelta(results),
      fidelityHeadline: args.skipFidelity ? null : aggregateFidelityHeadline(results),
    },
    results,
    diagnostics: {
      migratedOutputs: collectMigratedOutputDiagnostics(DIAGNOSTIC_STYLES),
    },
  };

  const json = `${JSON.stringify(report, null, 2)}\n`;
  if (args.out) fs.writeFileSync(path.resolve(args.out), json);
  if (args.markdown) fs.writeFileSync(path.resolve(args.markdown), buildMarkdown(report));
  if (args.json || (!args.out && !args.markdown)) process.stdout.write(json);
}

function collectMigratedOutputDiagnostics(styles) {
  const rows = [];
  for (const style of styles) {
    const row = { style };
    const migrated = migrateStyleToYaml(style);
    if (migrated.error) {
      row.error = migrated;
    } else {
      const { loaded, cleanup } = loadStyleFromYamlText(style, migrated.yaml);
      try {
        row.migrated = scoreYaml(loaded, migrated.yaml);
      } finally {
        cleanup();
      }
    }
    rows.push(row);
  }
  return rows;
}

if (require.main === module) {
  main().catch((err) => {
    console.error(`Error: ${err.message}`);
    process.exit(1);
  });
}

module.exports = {
  evaluateMinimizationAcceptance,
  normalizedEqual,
  strictSectionEquivalent,
  rowFidelityStatus,
  combinedFidelity,
  aggregateFidelityHeadline,
  FIDELITY_HEADLINE_THRESHOLD,
};
