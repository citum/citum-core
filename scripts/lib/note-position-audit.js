'use strict';

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');
const { execFileSync } = require('child_process');

const PROJECT_ROOT = path.resolve(__dirname, '..', '..');
const STYLES_DIR = path.join(PROJECT_ROOT, 'styles');
const EXPECTATIONS_PATH = path.join(PROJECT_ROOT, 'scripts', 'report-data', 'note-position-expectations.yaml');
const REFS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-note-positions.json');
const CITATIONS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-note-positions.json');
const DEFAULT_TIMEOUT_MS = 120000;
const REQUIRED_IDS = [
  'note-first',
  'note-ibid',
  'note-ibid-with-locator',
  'note-intervening',
  'note-subsequent',
  'note-subsequent-with-locator',
];

const CUSTOM_TAG_SCHEMA = yaml.DEFAULT_SCHEMA.extend([
  new yaml.Type('!custom', {
    kind: 'mapping',
    construct(data) {
      return data || {};
    },
  }),
]);
const RENDER_CACHE = new Map();

function readYaml(filePath) {
  return yaml.load(fs.readFileSync(filePath, 'utf8'), { schema: CUSTOM_TAG_SCHEMA }) || {};
}

function normalizeText(text) {
  return String(text || '').replace(/\s+/g, ' ').trim();
}

function validateExpectations(config) {
  if (!config || typeof config !== 'object' || Array.isArray(config)) {
    throw new Error('note-position-expectations.yaml must be an object');
  }
  if (config.version !== 1) {
    throw new Error('note-position-expectations.yaml version must be 1');
  }
  if (!config.profiles || typeof config.profiles !== 'object' || Array.isArray(config.profiles)) {
    throw new Error('note-position-expectations.yaml must define profiles');
  }
  if (!config.styles || typeof config.styles !== 'object' || Array.isArray(config.styles)) {
    throw new Error('note-position-expectations.yaml must define styles');
  }

  for (const [profileName, profile] of Object.entries(config.profiles)) {
    if (!profile || typeof profile !== 'object' || Array.isArray(profile)) {
      throw new Error(`note-position-expectations.yaml profiles.${profileName} must be an object`);
    }
    for (const key of ['ibid', 'subsequent']) {
      if (typeof profile.requires?.[key] !== 'boolean') {
        throw new Error(`note-position-expectations.yaml profiles.${profileName}.requires.${key} must be a boolean`);
      }
    }
    for (const key of ['lexical_ibid', 'immediate_falls_back_to_subsequent', 'distinct_subsequent']) {
      if (typeof profile.checks?.[key] !== 'boolean') {
        throw new Error(`note-position-expectations.yaml profiles.${profileName}.checks.${key} must be a boolean`);
      }
    }
  }

  for (const [styleName, styleEntry] of Object.entries(config.styles)) {
    if (!styleEntry || typeof styleEntry !== 'object' || Array.isArray(styleEntry)) {
      throw new Error(`note-position-expectations.yaml styles.${styleName} must be an object`);
    }
    if (typeof styleEntry.profile !== 'string' || !config.profiles[styleEntry.profile]) {
      throw new Error(`note-position-expectations.yaml styles.${styleName}.profile must reference a known profile`);
    }
  }

  return config;
}

function loadNotePositionExpectations(filePath = EXPECTATIONS_PATH) {
  return validateExpectations(readYaml(filePath));
}

function discoverNoteStyles(stylesDir = STYLES_DIR) {
  const styles = [];
  for (const entry of fs.readdirSync(stylesDir).filter((name) => name.endsWith('.yaml')).sort()) {
    const absolutePath = path.join(stylesDir, entry);
    const style = readYaml(absolutePath);
    if (style?.options?.processing === 'note') {
      styles.push({
        name: path.basename(entry, '.yaml'),
        path: absolutePath,
        style,
      });
    }
  }
  return styles;
}

function validateExpectationCoverage(noteStyles, expectations) {
  const declared = new Set(Object.keys(expectations.styles || {}));
  const discovered = new Set(noteStyles.map((style) => style.name));
  const missing = [...discovered].filter((name) => !declared.has(name)).sort();
  const extra = [...declared].filter((name) => !discovered.has(name)).sort();
  return { missing, extra };
}

function parseRenderedCitations(output) {
  const citations = {};
  let inCitations = false;
  for (const line of output.split('\n')) {
    if (line.includes('CITATIONS (From file):')) {
      inCitations = true;
      continue;
    }
    if (!inCitations) continue;
    const match = line.match(/^\s*\[([^\]]+)\]\s+(.+)$/);
    if (match) {
      citations[match[1]] = normalizeText(match[2]);
    }
  }
  return citations;
}

function renderFixtureForStyle(stylePath, refsFixture = REFS_FIXTURE, citationsFixture = CITATIONS_FIXTURE) {
  return renderFixtureForStyleWithOptions(stylePath, refsFixture, citationsFixture, {});
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
    timeout: DEFAULT_TIMEOUT_MS,
  });

  const builtBinary = cargoTargetDir
    ? path.join(cargoTargetDir, 'debug', 'citum')
    : path.join(PROJECT_ROOT, 'target', 'debug', 'citum');
  if (!fs.existsSync(builtBinary)) {
    throw new Error(`Expected Citum binary after build: ${builtBinary}`);
  }
  return builtBinary;
}

function createRenderCacheKey(binaryPath, stylePath, refsFixture, citationsFixture) {
  const parts = [binaryPath, stylePath, refsFixture, citationsFixture].map((filePath) => {
    const stats = fs.statSync(filePath);
    return `${path.resolve(filePath)}:${stats.mtimeMs}:${stats.size}`;
  });
  return parts.join('|');
}

function renderFixtureForStyleWithOptions(
  stylePath,
  refsFixture = REFS_FIXTURE,
  citationsFixture = CITATIONS_FIXTURE,
  options = {}
) {
  const binaryPath = resolveCitumBinary(options.citumBin || null);
  const cacheKey = createRenderCacheKey(binaryPath, stylePath, refsFixture, citationsFixture);
  if (RENDER_CACHE.has(cacheKey)) {
    return RENDER_CACHE.get(cacheKey);
  }

  const output = execFileSync(
    binaryPath,
    [
      'render',
      'refs',
      '-b',
      refsFixture,
      '-s',
      stylePath,
      '-c',
      citationsFixture,
      '--mode',
      'cite',
      '--show-keys',
    ],
    {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
      timeout: options.timeoutMs || DEFAULT_TIMEOUT_MS,
    }
  );
  const rendered = parseRenderedCitations(output);
  RENDER_CACHE.set(cacheKey, rendered);
  return rendered;
}

function normalizeAuditOptions(expectationOrOptions) {
  if (!expectationOrOptions) {
    return { expectationConfig: loadNotePositionExpectations() };
  }
  if (expectationOrOptions.version === 1 && expectationOrOptions.profiles && expectationOrOptions.styles) {
    return { expectationConfig: expectationOrOptions };
  }
  return {
    expectationConfig: expectationOrOptions.expectationConfig || loadNotePositionExpectations(),
    citumBin: expectationOrOptions.citumBin || null,
    timeoutMs: expectationOrOptions.timeoutMs || DEFAULT_TIMEOUT_MS,
  };
}

function auditNoteStyle(styleRecord, expectationOrOptions = null) {
  const options = normalizeAuditOptions(expectationOrOptions);
  const rendered = renderFixtureForStyleWithOptions(
    styleRecord.path,
    REFS_FIXTURE,
    CITATIONS_FIXTURE,
    options
  );
  return {
    style: styleRecord.name,
    ...evaluateNotePositionRender(
      styleRecord.name,
      styleRecord.style,
      rendered,
      options.expectationConfig
    ),
  };
}

function hasLexicalIbid(text) {
  return /\bibid\b/i.test(text || '');
}

function normalizeFixtureLocators(text) {
  return normalizeText(text).replace(/\b(105|205)\b/g, '__LOCATOR__');
}

function evaluateNotePositionRender(styleName, style, rendered, expectationConfig) {
  const styleExpectation = expectationConfig.styles[styleName];
  const profile = expectationConfig.profiles[styleExpectation.profile];
  const issues = [];
  const citationConfig = style.citation || {};

  for (const id of REQUIRED_IDS) {
    if (!rendered[id]) {
      issues.push({ kind: 'rendering-gap', message: `Missing rendered citation for fixture id: ${id}` });
    }
  }
  if (issues.length > 0) {
    return { status: 'rendering-gap', issues, profile: styleExpectation.profile };
  }

  if (profile.requires.ibid && !citationConfig.ibid) {
    issues.push({ kind: 'configuration-gap', message: 'Expected citation.ibid override is missing from style YAML.' });
  }
  if (profile.requires.subsequent && !citationConfig.subsequent) {
    issues.push({ kind: 'configuration-gap', message: 'Expected citation.subsequent override is missing from style YAML.' });
  }

  const first = rendered['note-first'];
  const ibid = rendered['note-ibid'];
  const ibidWithLocator = rendered['note-ibid-with-locator'];
  const subsequent = rendered['note-subsequent'];
  const subsequentWithLocator = rendered['note-subsequent-with-locator'];

  if (profile.checks.lexical_ibid) {
    if (!hasLexicalIbid(ibid)) {
      issues.push({ kind: 'rendering-gap', message: 'Immediate repeat should render lexical ibid.' });
    }
    if (!hasLexicalIbid(ibidWithLocator)) {
      issues.push({ kind: 'rendering-gap', message: 'Immediate repeat with locator should render lexical ibid.' });
    }
  } else if (hasLexicalIbid(ibid) || hasLexicalIbid(ibidWithLocator)) {
    issues.push({ kind: 'rendering-gap', message: 'Style should not render lexical ibid for immediate repeats.' });
  }

  if (profile.checks.immediate_falls_back_to_subsequent) {
    if (ibid !== subsequent) {
      issues.push({ kind: 'rendering-gap', message: 'Immediate repeat should fall back to the same rendering as subsequent repeat.' });
    }
    if (normalizeFixtureLocators(ibidWithLocator) !== normalizeFixtureLocators(subsequentWithLocator)) {
      issues.push({ kind: 'rendering-gap', message: 'Immediate repeat with locator should fall back to the same rendering as subsequent repeat with locator.' });
    }
  }

  if (profile.checks.distinct_subsequent && subsequent === first) {
    issues.push({ kind: 'rendering-gap', message: 'Subsequent citation should differ from the first full citation.' });
  }

  if (!subsequentWithLocator.includes('205')) {
    issues.push({ kind: 'rendering-gap', message: 'Subsequent citation with locator should preserve locator 205.' });
  }
  if (profile.checks.lexical_ibid && !ibidWithLocator.includes('105')) {
    issues.push({ kind: 'rendering-gap', message: 'Ibid-with-locator citation should preserve locator 105.' });
  }

  const status = issues.some((issue) => issue.kind === 'configuration-gap')
    ? 'configuration-gap'
    : issues.length > 0
      ? 'rendering-gap'
      : 'pass';

  return {
    status,
    issues,
    profile: styleExpectation.profile,
    rendered,
  };
}

function auditNoteStyle(styleRecord, expectationOrOptions = null) {
  const options = normalizeAuditOptions(expectationOrOptions);
  const rendered = renderFixtureForStyleWithOptions(
    styleRecord.path,
    REFS_FIXTURE,
    CITATIONS_FIXTURE,
    options
  );
  return {
    style: styleRecord.name,
    ...evaluateNotePositionRender(
      styleRecord.name,
      styleRecord.style,
      rendered,
      options.expectationConfig
    ),
  };
}

function summarizeAuditResults(results, coverage = { missing: [], extra: [] }) {
  const summary = {
    total: results.length,
    pass: 0,
    configurationGap: 0,
    renderingGap: 0,
    missingExpectations: coverage.missing.length,
    extraExpectations: coverage.extra.length,
  };
  for (const result of results) {
    if (result.status === 'pass') summary.pass += 1;
    if (result.status === 'configuration-gap') summary.configurationGap += 1;
    if (result.status === 'rendering-gap') summary.renderingGap += 1;
  }
  return summary;
}

function auditAllNoteStyles(options = {}) {
  const noteStyles = discoverNoteStyles(options.stylesDir || STYLES_DIR);
  const expectations = loadNotePositionExpectations(options.expectationsPath || EXPECTATIONS_PATH);
  const coverage = validateExpectationCoverage(noteStyles, expectations);
  const selected = options.styles?.length ? new Set(options.styles) : null;
  const results = noteStyles
    .filter((style) => !selected || selected.has(style.name))
    .map((style) =>
      auditNoteStyle(style, {
        expectationConfig: expectations,
        citumBin: options.citumBin || null,
        timeoutMs: options.timeoutMs || DEFAULT_TIMEOUT_MS,
      })
    );

  return {
    fixture: {
      refs: REFS_FIXTURE,
      citations: CITATIONS_FIXTURE,
    },
    coverage,
    results,
    summary: summarizeAuditResults(results, coverage),
  };
}

module.exports = {
  CITATIONS_FIXTURE,
  DEFAULT_TIMEOUT_MS,
  EXPECTATIONS_PATH,
  REFS_FIXTURE,
  STYLES_DIR,
  auditAllNoteStyles,
  auditNoteStyle,
  discoverNoteStyles,
  evaluateNotePositionRender,
  hasLexicalIbid,
  loadNotePositionExpectations,
  normalizeText,
  normalizeFixtureLocators,
  parseRenderedCitations,
  renderFixtureForStyleWithOptions,
  resolveCitumBinary,
  summarizeAuditResults,
  validateExpectationCoverage,
  validateExpectations,
};
