'use strict';

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');
const { execFileSync } = require('child_process');
const { resolveStyleData } = require('./verification-policy');

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
  if (config.version !== 3) {
    throw new Error('note-position-expectations.yaml version must be 3');
  }
  if (!config.regression_profiles || typeof config.regression_profiles !== 'object' || Array.isArray(config.regression_profiles)) {
    throw new Error('note-position-expectations.yaml must define regression_profiles');
  }
  if (!config.conformance_families || typeof config.conformance_families !== 'object' || Array.isArray(config.conformance_families)) {
    throw new Error('note-position-expectations.yaml must define conformance_families');
  }
  if (!config.styles || typeof config.styles !== 'object' || Array.isArray(config.styles)) {
    throw new Error('note-position-expectations.yaml must define styles');
  }

  for (const [profileName, profile] of Object.entries(config.regression_profiles)) {
    if (!profile || typeof profile !== 'object' || Array.isArray(profile)) {
      throw new Error(`note-position-expectations.yaml regression_profiles.${profileName} must be an object`);
    }
    for (const key of ['ibid', 'subsequent']) {
      if (typeof profile.requires?.[key] !== 'boolean') {
        throw new Error(`note-position-expectations.yaml regression_profiles.${profileName}.requires.${key} must be a boolean`);
      }
    }
    for (const key of ['lexical_ibid', 'immediate_falls_back_to_subsequent', 'distinct_subsequent']) {
      if (typeof profile.checks?.[key] !== 'boolean') {
        throw new Error(`note-position-expectations.yaml regression_profiles.${profileName}.checks.${key} must be a boolean`);
      }
    }
  }

  for (const [familyName, family] of Object.entries(config.conformance_families)) {
    if (!family || typeof family !== 'object' || Array.isArray(family)) {
      throw new Error(`note-position-expectations.yaml conformance_families.${familyName} must be an object`);
    }
    if (!['ibid', 'none'].includes(family.lexical_marker)) {
      throw new Error(`note-position-expectations.yaml conformance_families.${familyName}.lexical_marker must be "ibid" or "none"`);
    }
    for (const key of ['immediate_repeat_form', 'immediate_with_locator_form']) {
      if (!['marker', 'shortened-note'].includes(family[key])) {
        throw new Error(`note-position-expectations.yaml conformance_families.${familyName}.${key} must be "marker" or "shortened-note"`);
      }
    }
    if (!['base', 'subsequent'].includes(family.shortened_note_source)) {
      throw new Error(`note-position-expectations.yaml conformance_families.${familyName}.shortened_note_source must be "base" or "subsequent"`);
    }
    if (typeof family.distinct_subsequent !== 'boolean') {
      throw new Error(`note-position-expectations.yaml conformance_families.${familyName}.distinct_subsequent must be a boolean`);
    }
    if (
      family.note_start_text_case !== undefined
      && !['capitalize-first', 'lowercase'].includes(family.note_start_text_case)
    ) {
      throw new Error(
        `note-position-expectations.yaml conformance_families.${familyName}.note_start_text_case must be "capitalize-first" or "lowercase"`
      );
    }
    if (!Array.isArray(family.unresolved)) {
      throw new Error(`note-position-expectations.yaml conformance_families.${familyName}.unresolved must be an array`);
    }
  }

  for (const [styleName, styleEntry] of Object.entries(config.styles)) {
    if (!styleEntry || typeof styleEntry !== 'object' || Array.isArray(styleEntry)) {
      throw new Error(`note-position-expectations.yaml styles.${styleName} must be an object`);
    }
    if (
      typeof styleEntry.regression_profile !== 'string'
      || !config.regression_profiles[styleEntry.regression_profile]
    ) {
      throw new Error(`note-position-expectations.yaml styles.${styleName}.regression_profile must reference a known regression profile`);
    }
    if (
      typeof styleEntry.conformance_family !== 'string'
      || !config.conformance_families[styleEntry.conformance_family]
    ) {
      throw new Error(`note-position-expectations.yaml styles.${styleName}.conformance_family must reference a known conformance family`);
    }
  }

  return config;
}

function loadNotePositionExpectations(filePath = EXPECTATIONS_PATH) {
  return validateExpectations(readYaml(filePath));
}

function discoverNoteStyles(stylesDir = STYLES_DIR) {
  const styles = [];
  const embeddedDir = path.join(stylesDir, 'embedded');

  const scan = (dir) => {
    if (!fs.existsSync(dir)) return;
    for (const entry of fs.readdirSync(dir).filter((name) => name.endsWith('.yaml')).sort()) {
      const absolutePath = path.join(dir, entry);
      const style = resolveStyleData(readYaml(absolutePath));
      if (style?.options?.processing === 'note') {
        styles.push({
          name: path.basename(entry, '.yaml'),
          path: absolutePath,
          style,
        });
      }
    }
  };

  scan(stylesDir);
  scan(embeddedDir);

  return styles.sort((a, b) => a.name.localeCompare(b.name));
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
  if (
    expectationOrOptions.version === 3
    && expectationOrOptions.regression_profiles
    && expectationOrOptions.conformance_families
    && expectationOrOptions.styles
  ) {
    return { expectationConfig: expectationOrOptions };
  }
  return {
    expectationConfig: expectationOrOptions.expectationConfig || loadNotePositionExpectations(),
    citumBin: expectationOrOptions.citumBin || null,
    timeoutMs: expectationOrOptions.timeoutMs || DEFAULT_TIMEOUT_MS,
  };
}

function hasLexicalIbid(text) {
  return /\bibid\b/i.test(text || '');
}

function matchesLeadingTextCase(text, expectedCase) {
  const normalized = normalizeText(text);
  const match = normalized.match(/\p{L}/u);
  if (!match) {
    return false;
  }
  const first = match[0];
  if (expectedCase === 'capitalize-first') {
    return first === first.toUpperCase() && first !== first.toLowerCase();
  }
  if (expectedCase === 'lowercase') {
    return first === first.toLowerCase() && first !== first.toUpperCase();
  }
  return false;
}

function styleNoteStartTextCase(citationSpec) {
  if (!citationSpec || typeof citationSpec !== 'object') {
    return null;
  }
  return citationSpec.note_start_text_case || citationSpec['note-start-text-case'] || null;
}

function normalizeFixtureLocators(text) {
  return normalizeText(text).replace(/\b(105|205)\b/g, '__LOCATOR__');
}

function compareRenderedForms(left, right) {
  return normalizeFixtureLocators(left) === normalizeFixtureLocators(right);
}

function ensureRequiredFixtureIds(rendered) {
  const issues = [];
  for (const id of REQUIRED_IDS) {
    if (!rendered[id]) {
      issues.push({ kind: 'rendering-gap', message: `Missing rendered citation for fixture id: ${id}` });
    }
  }
  return issues;
}

function evaluateRegressionLayer(styleName, style, rendered, expectationConfig) {
  const styleExpectation = expectationConfig.styles[styleName];
  const profile = expectationConfig.regression_profiles[styleExpectation.regression_profile];
  const issues = ensureRequiredFixtureIds(rendered);
  const citationConfig = style.citation || {};

  if (issues.length > 0) {
    return { status: 'rendering-gap', issues, profile: styleExpectation.regression_profile };
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
    if (!compareRenderedForms(ibid, subsequent)) {
      issues.push({ kind: 'rendering-gap', message: 'Immediate repeat should fall back to the same rendering as subsequent repeat.' });
    }
    if (!compareRenderedForms(ibidWithLocator, subsequentWithLocator)) {
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
    profile: styleExpectation.regression_profile,
  };
}

function evaluateConformanceLayer(styleName, style, rendered, expectationConfig) {
  const styleExpectation = expectationConfig.styles[styleName];
  const family = expectationConfig.conformance_families[styleExpectation.conformance_family];
  const issues = ensureRequiredFixtureIds(rendered).map((issue) => ({
    kind: 'rendering-gap',
    message: issue.message,
  }));

  if (issues.length > 0) {
    return {
      status: 'gap',
      issues,
      family: styleExpectation.conformance_family,
      unresolved: family.unresolved,
    };
  }

  const citationConfig = style.citation || {};
  const first = rendered['note-first'];
  const ibid = rendered['note-ibid'];
  const ibidWithLocator = rendered['note-ibid-with-locator'];
  const subsequent = rendered['note-subsequent'];
  const subsequentWithLocator = rendered['note-subsequent-with-locator'];

  if (family.lexical_marker === 'ibid') {
    if (!citationConfig.ibid) {
      issues.push({ kind: 'style-gap', message: 'Conformance family expects a lexical ibid override in style YAML.' });
    }
    if (!hasLexicalIbid(ibid) || !hasLexicalIbid(ibidWithLocator)) {
      issues.push({ kind: 'rendering-gap', message: 'Conformance family expects lexical ibid output for immediate repeats.' });
    }
    if (family.note_start_text_case) {
      if (styleNoteStartTextCase(citationConfig.ibid) !== family.note_start_text_case) {
        issues.push({
          kind: 'style-gap',
          message: `Conformance family expects citation.ibid.note_start_text_case=${family.note_start_text_case}.`,
        });
      }
      if (!matchesLeadingTextCase(ibid, family.note_start_text_case)) {
        issues.push({
          kind: 'rendering-gap',
          message: `Conformance family expects note-start ibid output with ${family.note_start_text_case} case.`,
        });
      }
      if (!matchesLeadingTextCase(ibidWithLocator, family.note_start_text_case)) {
        issues.push({
          kind: 'rendering-gap',
          message: `Conformance family expects note-start ibid-with-locator output with ${family.note_start_text_case} case.`,
        });
      }
    }
  } else if (hasLexicalIbid(ibid) || hasLexicalIbid(ibidWithLocator)) {
    issues.push({ kind: 'rendering-gap', message: 'Conformance family does not allow lexical ibid for immediate repeats.' });
  }

  if (family.immediate_repeat_form === 'shortened-note' && !compareRenderedForms(ibid, subsequent)) {
    issues.push({ kind: 'rendering-gap', message: 'Conformance family expects immediate repeats to reuse the shortened-note form.' });
  }

  if (family.immediate_with_locator_form === 'shortened-note' && !compareRenderedForms(ibidWithLocator, subsequentWithLocator)) {
    issues.push({ kind: 'rendering-gap', message: 'Conformance family expects immediate repeats with locator to reuse the shortened-note form.' });
  }

  if (family.shortened_note_source === 'subsequent' && !citationConfig.subsequent) {
    issues.push({ kind: 'style-gap', message: 'Conformance family expects shortened-note behavior in citation.subsequent.' });
  }

  if (family.shortened_note_source === 'base' && citationConfig.subsequent) {
    issues.push({ kind: 'style-gap', message: 'Conformance family expects the base citation to already be the shortened-note form.' });
  }

  if (family.distinct_subsequent) {
    if (subsequent === first) {
      issues.push({ kind: 'rendering-gap', message: 'Conformance family expects later repeats to differ from the first full note.' });
    }
  } else if (!compareRenderedForms(subsequent, first)) {
    issues.push({ kind: 'rendering-gap', message: 'Conformance family expects later repeats to reuse the base shortened-note form.' });
  }

  if (!subsequentWithLocator.includes('205')) {
    issues.push({ kind: 'rendering-gap', message: 'Conformance family expects locator-preserving subsequent short notes.' });
  }
  if (family.lexical_marker === 'ibid' && family.immediate_with_locator_form === 'marker' && !ibidWithLocator.includes('105')) {
    issues.push({ kind: 'rendering-gap', message: 'Conformance family expects locator-preserving lexical ibid output.' });
  }

  return {
    status: issues.length > 0 ? 'gap' : 'pass',
    issues,
    family: styleExpectation.conformance_family,
    unresolved: family.unresolved,
  };
}

function evaluateNotePositionRender(styleName, style, rendered, expectationConfig) {
  const regression = evaluateRegressionLayer(styleName, style, rendered, expectationConfig);
  const conformance = evaluateConformanceLayer(styleName, style, rendered, expectationConfig);

  return {
    status: regression.status,
    issues: regression.issues,
    profile: regression.profile,
    rendered,
    regression,
    conformance,
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
    regression: {
      pass: 0,
      configurationGap: 0,
      renderingGap: 0,
    },
    conformance: {
      pass: 0,
      gap: 0,
      unresolved: 0,
    },
    missingExpectations: coverage.missing.length,
    extraExpectations: coverage.extra.length,
  };

  for (const result of results) {
    if (result.regression.status === 'pass') summary.regression.pass += 1;
    if (result.regression.status === 'configuration-gap') summary.regression.configurationGap += 1;
    if (result.regression.status === 'rendering-gap') summary.regression.renderingGap += 1;

    if (result.conformance.status === 'pass') summary.conformance.pass += 1;
    if (result.conformance.status === 'gap') summary.conformance.gap += 1;
    if (Array.isArray(result.conformance.unresolved) && result.conformance.unresolved.length > 0) {
      summary.conformance.unresolved += 1;
    }
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
  compareRenderedForms,
  discoverNoteStyles,
  evaluateConformanceLayer,
  evaluateNotePositionRender,
  evaluateRegressionLayer,
  hasLexicalIbid,
  loadNotePositionExpectations,
  normalizeText,
  normalizeFixtureLocators,
  parseRenderedCitations,
  matchesLeadingTextCase,
  renderFixtureForStyle,
  renderFixtureForStyleWithOptions,
  styleNoteStartTextCase,
  resolveCitumBinary,
  summarizeAuditResults,
  validateExpectationCoverage,
  validateExpectations,
};
