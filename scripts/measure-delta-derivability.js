#!/usr/bin/env node
/*
 * Delta-derivability sweep for citum-migrate.
 *
 * Question: what fraction of independent CSL 1.0 styles that are "near
 * clones" of a builtin/embedded parent (0.80 <= similarity < 0.98, per
 * `find-alias-candidates.js --threshold 0.80`) can be expressed as a small
 * `extends` wrapper over that parent at fidelity >= their current
 * standalone synthesized conversion?
 *
 * For each (candidate, target) near-clone pair:
 *   1. Standalone: migrate the candidate's legacy CSL normally
 *      (`citum-migrate <csl>`), score combined strict citation+bibliography
 *      fidelity against citeproc-js.
 *   2. Delta candidate: migrate with `--family-candidate <target>
 *      --minimize-wrapper`, which forces promotion of `target` as the
 *      parent and emits a minimal info+extends wrapper (see
 *      `LineageResolver::apply_to_migrated_style_minimized` in
 *      crates/citum-migrate/src/lineage.rs). Score the wrapper the same way.
 *   3. verdict is `delta-expressible` iff wrapper fidelity >= standalone
 *      fidelity.
 *
 * Fidelity scoring reuses `combinedFidelity`/`rowFidelityStatus` exported by
 * report-migrate-sqi.js so numbers are directly comparable to the scorecard.
 * Rendering goes through `scripts/oracle.js`, which shells out to the real
 * `citum render` CLI; that CLI resolves `extends: <id>` against the
 * registry-backed embedded parents compiled into the binary
 * (crates/citum-schema-style/embedded/styles/), so no additional plumbing is
 * needed to render an extends-wrapper style.
 *
 * Usage:
 *   node scripts/measure-delta-derivability.js --pairs <tsv> --corpus random --out scripts/report-data/delta-derivability-random100-<date>.tsv
 *   node scripts/measure-delta-derivability.js --pairs <tsv> --corpus styles --limit 5
 */

'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync, spawn } = require('child_process');
const yaml = require('js-yaml');

const { combinedFidelity, rowFidelityStatus } = require('./report-migrate-sqi.js');
const { classifyCitationFormat, stratifiedSample } = require('./lib/corpus-sample.js');

const WORKSPACE_ROOT = path.resolve(__dirname, '..');
const LEGACY_DIR = path.join(WORKSPACE_ROOT, 'styles-legacy');
const STYLES_DIR = path.join(WORKSPACE_ROOT, 'styles');

let MIGRATE_BIN = null;
const MIGRATE_PROFILE = 'release-unwind';

const DEFAULT_RANDOM_SEED = 20260610;
const DEFAULT_RANDOM_SAMPLE = 100;

function todayDate() {
  const now = new Date();
  return now.toISOString().split('T')[0];
}

const DEFAULT_OUT = path.join(
  WORKSPACE_ROOT,
  'scripts',
  'report-data',
  `delta-derivability-${todayDate()}.tsv`
);

function printHelp() {
  console.log(`
Usage: node scripts/measure-delta-derivability.js --pairs <tsv> --corpus random|styles [options]

Options:
  --pairs <path>      Input TSV from find-alias-candidates.js (required). Only
                       rows with band=near-clone are considered.
  --corpus <mode>     random: restrict to the seeded random-100 corpus
                       (stratifiedSample, seed ${DEFAULT_RANDOM_SEED}).
                       styles: restrict to candidates with a checked-in
                       styles/*.yaml (basename or info.source csl-id match).
  --limit N           Cap the number of pairs processed (after corpus filter).
  --concurrency N     Max pairs measured in parallel (default: 2). Each pair
                       spawns citum-migrate plus in-process rendering; keep low
                       on memory-constrained machines.
  --out <path>        TSV output path (default: ${DEFAULT_OUT}).
                       A JSON summary is written alongside it (.json).
  -h, --help          Show this help.
`);
}

function parseArgs(argv) {
  const args = {
    pairs: null,
    corpus: null,
    limit: null,
    concurrency: 2,
    out: DEFAULT_OUT,
  };
  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--pairs' && argv[i + 1]) {
      args.pairs = path.resolve(argv[++i]);
    } else if (arg === '--corpus' && argv[i + 1]) {
      args.corpus = argv[++i];
    } else if (arg === '--limit' && argv[i + 1]) {
      args.limit = Number(argv[++i]);
      if (!Number.isInteger(args.limit) || args.limit <= 0) {
        throw new Error('--limit must be a positive integer');
      }
    } else if (arg === '--concurrency' && argv[i + 1]) {
      args.concurrency = Number(argv[++i]);
      if (!Number.isInteger(args.concurrency) || args.concurrency <= 0) {
        throw new Error('--concurrency must be a positive integer');
      }
    } else if (arg === '--out' && argv[i + 1]) {
      args.out = path.resolve(argv[++i]);
    } else if (arg === '-h' || arg === '--help') {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (!args.pairs) throw new Error('--pairs is required');
  if (args.corpus !== 'random' && args.corpus !== 'styles') {
    throw new Error("--corpus must be 'random' or 'styles'");
  }
  return args;
}

// -- Migrate binary (mirrors scripts/report-migrate-sqi.js::migrateBinary) --
//
// Must be built with panic = "unwind" (the release-unwind profile), not the
// workspace-default release profile which sets panic = "abort" and would
// turn a single panicking candidate into a SIGABRT for the whole run. See
// the docstring on migrateBinary() in report-migrate-sqi.js for the full
// rationale.
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

/** Run async `worker(item, index)` over `items` with at most `limit` in flight. */
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

/** Migrate a style standalone (no family-candidate routing). */
async function migrateStandalone(styleName) {
  const cslPath = path.join(LEGACY_DIR, `${styleName}.csl`);
  if (!fs.existsSync(cslPath)) {
    return { error: 'missing_legacy_style', details: cslPath };
  }
  const proc = await spawnAsync(migrateBinary(), [cslPath], { cwd: WORKSPACE_ROOT });
  if (proc.status !== 0) {
    return { error: 'migrate_failed', details: (proc.stderr || '').trim() || `exit=${proc.status}` };
  }
  return { yaml: proc.stdout };
}

/** Migrate a style forcing `target` as the family-candidate parent, emitting a minimal wrapper. */
async function migrateWrapper(styleName, targetId) {
  const cslPath = path.join(LEGACY_DIR, `${styleName}.csl`);
  if (!fs.existsSync(cslPath)) {
    return { error: 'missing_legacy_style', details: cslPath };
  }
  const proc = await spawnAsync(
    migrateBinary(),
    ['--family-candidate', targetId, '--minimize-wrapper', cslPath],
    { cwd: WORKSPACE_ROOT }
  );
  if (proc.status !== 0) {
    return { error: 'migrate_failed', details: (proc.stderr || '').trim() || `exit=${proc.status}` };
  }
  return { yaml: proc.stdout };
}

/**
 * Render + score a pre-built YAML against citeproc-js via scripts/oracle.js,
 * exactly the path report-migrate-sqi.js uses for minimization acceptance
 * (see oracleForYaml there). `citum render` resolves `extends:` against the
 * registry-backed embedded parents compiled into the binary, so a minimal
 * extends-wrapper renders the same way a standalone style would.
 */
async function oracleForYaml(styleName, yamlText) {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-delta-oracle-'));
  const yamlPath = path.join(tmpDir, `${styleName}.yaml`);
  try {
    fs.writeFileSync(yamlPath, yamlText);
    const proc = await spawnAsync(
      'node',
      [path.join(WORKSPACE_ROOT, 'scripts', 'oracle.js'), yamlPath, '--json'],
      { cwd: WORKSPACE_ROOT }
    );
    if (!proc.stdout) return { error: 'oracle_no_output', details: (proc.stderr || '').trim() };
    const parsed = JSON.parse(proc.stdout);
    if (parsed.error) return { error: 'oracle_error', details: parsed.reason || parsed.error };
    if (!parsed.citations && !parsed.bibliography) return { error: 'oracle_empty' };
    return { citations: parsed.citations || null, bibliography: parsed.bibliography || null };
  } catch (err) {
    return { error: 'oracle_exception', details: err.message };
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

/** Build a report-migrate-sqi.js-shaped `row` so combinedFidelity/rowFidelityStatus apply unchanged. */
function toFidelityRow(migrated, oracleResult) {
  if (migrated.error) return { error: migrated };
  if (oracleResult.error) return { fidelity: { error: oracleResult.error, details: oracleResult.details } };
  return { fidelity: { citations: oracleResult.citations, bibliography: oracleResult.bibliography, error: null } };
}

function stripCustomYamlTags(yamlText) {
  // citum-migrate emits serde-tagged enum variants like `processing: !custom`.
  // js-yaml rejects unknown tags; drop the inline tag verb only.
  return yamlText.replace(/(:\s)![A-Za-z][A-Za-z0-9_-]*(\s|$)/g, '$1$2');
}

function wrapperTopLevelExtraKeyCount(yamlText) {
  try {
    const data = yaml.load(stripCustomYamlTags(yamlText));
    if (!data || typeof data !== 'object') return null;
    return Object.keys(data).filter((k) => k !== 'info' && k !== 'extends' && k !== 'extends-pin').length;
  } catch {
    return null;
  }
}

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

/** Same corpus draw report-migrate-sqi.js uses for `--corpus random`. */
function randomCorpusStyleSet() {
  const classified = enumerateIndependentStyles().map((style) => ({
    style,
    styleClass: classifyLegacyStyle(style),
  }));
  const drawn = stratifiedSample(classified, {
    sampleSize: DEFAULT_RANDOM_SAMPLE,
    seed: DEFAULT_RANDOM_SEED,
  });
  return {
    set: new Set(drawn.sample.map((entry) => entry.style)),
    sampleMeta: {
      seed: DEFAULT_RANDOM_SEED,
      requested: DEFAULT_RANDOM_SAMPLE,
      population: drawn.population,
      strata: drawn.strata,
      allocation: drawn.allocation,
    },
  };
}

/**
 * Legacy CSL ids covered by a checked-in styles/*.yaml. Each converted style
 * maps back to its legacy source either by basename (styles/<id>.yaml <->
 * styles-legacy/<id>.csl) or via `info.source.csl-id` (a Zotero-style URL
 * whose last path segment is the legacy CSL id).
 */
function stylesCorpusIdSet() {
  const ids = new Set();
  const entries = fs.readdirSync(STYLES_DIR, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isDirectory()) continue; // skip experimental/
    if (!entry.name.endsWith('.yaml')) continue;
    const basename = path.basename(entry.name, '.yaml');
    ids.add(basename);
    try {
      const filePath = path.join(STYLES_DIR, entry.name);
      const stat = fs.lstatSync(filePath);
      if (stat.isSymbolicLink()) continue; // skip the embedded symlink
      const data = yaml.load(stripCustomYamlTags(fs.readFileSync(filePath, 'utf8')));
      const cslId = data?.info?.source?.['csl-id'];
      if (typeof cslId === 'string' && cslId.length > 0) {
        const segments = cslId.split('/').filter(Boolean);
        const last = segments[segments.length - 1];
        if (last) ids.add(last);
      }
    } catch {
      // Non-fatal: fall back to basename-only matching for this style.
    }
  }
  return ids;
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

function readPairsTsv(tsvPath) {
  const content = fs.readFileSync(tsvPath, 'utf8');
  const lines = content.split('\n').filter((l) => l.trim().length > 0);
  if (lines.length === 0) return [];
  const header = lines[0].split('\t');
  const rows = [];
  for (let i = 1; i < lines.length; i++) {
    const cols = lines[i].split('\t');
    const row = Object.fromEntries(header.map((h, j) => [h, cols[j]]));
    row.similarity = Number(row.similarity);
    rows.push(row);
  }
  return rows;
}

function median(sortedValues) {
  const n = sortedValues.length;
  if (n === 0) return null;
  const mid = Math.floor(n / 2);
  return n % 2 === 0 ? (sortedValues[mid - 1] + sortedValues[mid]) / 2 : sortedValues[mid];
}

function mean(values) {
  if (values.length === 0) return null;
  return values.reduce((a, b) => a + b, 0) / values.length;
}

async function measurePair(pair) {
  const { candidate_id: candidateId, best_target: targetId, similarity } = pair;

  const record = {
    candidate_id: candidateId,
    target_id: targetId,
    similarity,
    standalone_fidelity: null,
    wrapper_fidelity: null,
    wrapper_bytes: null,
    wrapper_key_count: null,
    verdict: 'error',
    detail: '',
  };

  const [standaloneMigrated, wrapperMigrated] = await Promise.all([
    migrateStandalone(candidateId),
    migrateWrapper(candidateId, targetId),
  ]);

  if (standaloneMigrated.error) {
    record.detail = `standalone-migrate: ${standaloneMigrated.details || standaloneMigrated.error}`;
    return record;
  }
  if (wrapperMigrated.error) {
    record.detail = `wrapper-migrate: ${wrapperMigrated.details || wrapperMigrated.error}`;
    return record;
  }

  record.wrapper_bytes = Buffer.byteLength(wrapperMigrated.yaml, 'utf8');
  record.wrapper_key_count = wrapperTopLevelExtraKeyCount(wrapperMigrated.yaml);

  const [standaloneOracle, wrapperOracle] = await Promise.all([
    oracleForYaml(candidateId, standaloneMigrated.yaml),
    oracleForYaml(`${candidateId}-as-${targetId}`, wrapperMigrated.yaml),
  ]);

  const standaloneRow = toFidelityRow(standaloneMigrated, standaloneOracle);
  const wrapperRow = toFidelityRow(wrapperMigrated, wrapperOracle);

  if (rowFidelityStatus(standaloneRow) !== 'ok') {
    record.detail = `standalone-oracle: ${standaloneOracle.error || 'unknown'}${standaloneOracle.details ? ` (${standaloneOracle.details})` : ''}`;
    return record;
  }
  if (rowFidelityStatus(wrapperRow) !== 'ok') {
    record.detail = `wrapper-oracle: ${wrapperOracle.error || 'unknown'}${wrapperOracle.details ? ` (${wrapperOracle.details})` : ''}`;
    return record;
  }

  record.standalone_fidelity = combinedFidelity(standaloneRow);
  record.wrapper_fidelity = combinedFidelity(wrapperRow);
  record.verdict = record.wrapper_fidelity >= record.standalone_fidelity
    ? 'delta-expressible'
    : 'not-delta-expressible';
  return record;
}

function writeOutputs(args, records, bandCounts, corpusMeta) {
  const header = [
    'candidate_id',
    'target_id',
    'similarity',
    'standalone_fidelity',
    'wrapper_fidelity',
    'wrapper_bytes',
    'wrapper_key_count',
    'verdict',
    'detail',
  ];
  const fmt = (value) => (typeof value === 'number' ? value.toFixed(4) : value ?? '');
  const lines = [header.join('\t')];
  for (const record of records) {
    lines.push([
      record.candidate_id,
      record.target_id,
      fmt(record.similarity),
      fmt(record.standalone_fidelity),
      fmt(record.wrapper_fidelity),
      record.wrapper_bytes ?? '',
      record.wrapper_key_count ?? '',
      record.verdict,
      record.detail || '',
    ].join('\t'));
  }
  fs.mkdirSync(path.dirname(args.out), { recursive: true });
  fs.writeFileSync(args.out, lines.join('\n') + '\n');

  const verdictCounts = { 'delta-expressible': 0, 'not-delta-expressible': 0, error: 0 };
  for (const record of records) verdictCounts[record.verdict] += 1;

  const comparable = records.filter(
    (r) => r.standalone_fidelity != null && r.wrapper_fidelity != null
  );
  const deltas = comparable
    .map((r) => r.wrapper_fidelity - r.standalone_fidelity)
    .sort((a, b) => a - b);
  const wrapperBytes = records
    .map((r) => r.wrapper_bytes)
    .filter((v) => typeof v === 'number')
    .sort((a, b) => a - b);
  const wrapperKeyCounts = records
    .map((r) => r.wrapper_key_count)
    .filter((v) => typeof v === 'number')
    .sort((a, b) => a - b);

  const nonErrorTotal = verdictCounts['delta-expressible'] + verdictCounts['not-delta-expressible'];

  const summary = {
    generated: new Date().toISOString(),
    commit: gitCommit(),
    pairsFile: args.pairs,
    corpus: args.corpus,
    ...(corpusMeta ? { sample: corpusMeta } : {}),
    bandDistribution: bandCounts,
    counts: {
      pairsConsidered: records.length,
      verdictCounts,
      shareDeltaExpressibleExcludingErrors: nonErrorTotal
        ? Number(((verdictCounts['delta-expressible'] / nonErrorTotal) * 100).toFixed(1))
        : null,
      shareDeltaExpressibleOfAllConsidered: records.length
        ? Number(((verdictCounts['delta-expressible'] / records.length) * 100).toFixed(1))
        : null,
    },
    fidelityDelta: comparable.length
      ? {
          n: comparable.length,
          mean: Number((mean(deltas) * 100).toFixed(2)),
          median: Number((median(deltas) * 100).toFixed(2)),
        }
      : null,
    wrapperSize: {
      bytes: wrapperBytes.length
        ? { n: wrapperBytes.length, mean: Math.round(mean(wrapperBytes)), median: median(wrapperBytes), min: wrapperBytes[0], max: wrapperBytes[wrapperBytes.length - 1] }
        : null,
      extraTopLevelKeys: wrapperKeyCounts.length
        ? { n: wrapperKeyCounts.length, mean: Number(mean(wrapperKeyCounts).toFixed(2)), median: median(wrapperKeyCounts), min: wrapperKeyCounts[0], max: wrapperKeyCounts[wrapperKeyCounts.length - 1] }
        : null,
    },
    errors: records
      .filter((r) => r.verdict === 'error')
      .map((r) => ({ candidate_id: r.candidate_id, target_id: r.target_id, detail: r.detail })),
  };

  const jsonPath = args.out.replace(/\.tsv$/, '') + (args.out.endsWith('.tsv') ? '.json' : '.summary.json');
  fs.writeFileSync(jsonPath, `${JSON.stringify(summary, null, 2)}\n`);

  console.error(`TSV written to: ${args.out}`);
  console.error(`JSON summary written to: ${jsonPath}`);
  console.error(
    `Pairs considered: ${records.length} | delta-expressible: ${verdictCounts['delta-expressible']} | not: ${verdictCounts['not-delta-expressible']} | errors: ${verdictCounts.error}`
  );
}

async function main() {
  const args = parseArgs(process.argv);
  migrateBinary();

  const allPairs = readPairsTsv(args.pairs);
  const bandCounts = { alias: 0, 'near-clone': 0, other: 0 };
  for (const row of allPairs) {
    if (row.band === 'alias') bandCounts.alias += 1;
    else if (row.band === 'near-clone') bandCounts['near-clone'] += 1;
    else bandCounts.other += 1;
  }

  let nearClonePairs = allPairs.filter((row) => row.band === 'near-clone');

  let corpusMeta = null;
  if (args.corpus === 'random') {
    const { set, sampleMeta } = randomCorpusStyleSet();
    corpusMeta = sampleMeta;
    nearClonePairs = nearClonePairs.filter((row) => set.has(row.candidate_id));
  } else {
    const set = stylesCorpusIdSet();
    nearClonePairs = nearClonePairs.filter((row) => set.has(row.candidate_id));
  }

  if (args.limit) nearClonePairs = nearClonePairs.slice(0, args.limit);

  console.error(`Measuring ${nearClonePairs.length} near-clone pairs (corpus=${args.corpus})...`);

  const concurrency = Math.max(1, Math.min(args.concurrency, nearClonePairs.length));
  const records = await runPool(nearClonePairs, concurrency, (pair) => measurePair(pair));

  writeOutputs(args, records, bandCounts, corpusMeta);
}

if (require.main === module) {
  main().catch((err) => {
    console.error(`Error: ${err.message}`);
    process.exit(1);
  });
}

module.exports = {
  wrapperTopLevelExtraKeyCount,
  stylesCorpusIdSet,
  randomCorpusStyleSet,
  readPairsTsv,
};
