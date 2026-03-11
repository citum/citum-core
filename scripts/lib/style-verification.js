'use strict';

const path = require('path');
const {
  loadFixtureSufficiency,
  loadVerificationPolicy,
  resolveFixtureSufficiency,
  resolveVerificationPolicy,
} = require('./verification-policy');

const PROJECT_ROOT = path.resolve(__dirname, '..', '..');
const STYLES_DIR = path.join(PROJECT_ROOT, 'styles');
const LEGACY_STYLES_DIR = path.join(PROJECT_ROOT, 'styles-legacy');
const FIXTURES_DIR = path.join(PROJECT_ROOT, 'tests', 'fixtures');

const LEGACY_SOURCE_OVERRIDES = {
  'apa-7th': 'apa',
  'din-alphanumeric': 'din-1505-2-alphanumeric',
  'gost-r-7-0-5-2008-author-date': 'gost-r-7-0-5-2008',
};

const DEFAULT_REFS_FIXTURE = path.join(FIXTURES_DIR, 'references-expanded.json');
const DEFAULT_CITATIONS_FIXTURE = path.join(FIXTURES_DIR, 'citations-expanded.json');
const NOTE_CITATIONS_FIXTURE = path.join(FIXTURES_DIR, 'citations-note-expanded.json');

const FIXTURE_SET_REFS = {
  'compound-numeric': path.join(FIXTURES_DIR, 'references-compound-numeric-family.json'),
  'physics-numeric': path.join(FIXTURES_DIR, 'references-physics-numeric.json'),
  'author-date': path.join(FIXTURES_DIR, 'references-author-date.json'),
  'humanities-note': path.join(FIXTURES_DIR, 'references-humanities-note.json'),
  'note-positions': path.join(FIXTURES_DIR, 'references-note-positions.json'),
  'legal': path.join(FIXTURES_DIR, 'references-legal.json'),
  'csl-m-adapted': path.join(FIXTURES_DIR, 'references-csl-m-adapted.json'),
};

const FIXTURE_SET_CITATIONS = {
  'compound-numeric': path.join(FIXTURES_DIR, 'citations-compound-numeric.json'),
  'physics-numeric': path.join(FIXTURES_DIR, 'citations-physics-numeric.json'),
  'author-date': path.join(FIXTURES_DIR, 'citations-author-date.json'),
  'humanities-note': path.join(FIXTURES_DIR, 'citations-humanities-note.json'),
  'note-positions': path.join(FIXTURES_DIR, 'citations-note-positions.json'),
  'legal': path.join(FIXTURES_DIR, 'citations-legal.json'),
  'csl-m-adapted': path.join(FIXTURES_DIR, 'citations-csl-m-adapted.json'),
};

function inferLegacySourceName(styleName, styleData = null) {
  if (LEGACY_SOURCE_OVERRIDES[styleName]) {
    return LEGACY_SOURCE_OVERRIDES[styleName];
  }

  const sourceId = styleData?.info?.source?.['csl-id'];
  const match = typeof sourceId === 'string'
    ? sourceId.match(/zotero\.org\/styles\/([^/?#]+)/i)
    : null;

  return match?.[1] || styleName;
}

function resolveDefaultCitationFixture(styleFormat) {
  return styleFormat === 'note' ? NOTE_CITATIONS_FIXTURE : DEFAULT_CITATIONS_FIXTURE;
}

function getAdditionalFixtureSetNames(fixtureSets = []) {
  return fixtureSets.filter(
    (setName) =>
      setName !== 'core'
      && setName !== 'note'
      && setName !== 'note-positions'
      && FIXTURE_SET_REFS[setName]
  );
}

function getFixtureFiles(setName) {
  const refsFixture = FIXTURE_SET_REFS[setName];
  const citationsFixture = FIXTURE_SET_CITATIONS[setName];
  if (!refsFixture || !citationsFixture) {
    return null;
  }
  return { refsFixture, citationsFixture };
}

function getEffectiveVerificationScopes(stylePolicy, hasBibliography) {
  const scopes = Array.isArray(stylePolicy?.scopes) ? [...stylePolicy.scopes] : [];
  if (hasBibliography !== false) {
    return scopes;
  }

  const filtered = scopes.filter((scope) => scope !== 'bibliography');
  return filtered.length > 0 ? filtered : ['citation'];
}

function isProjectStylePath(yamlPath) {
  const absolutePath = path.resolve(yamlPath);
  return path.dirname(absolutePath) === STYLES_DIR;
}

function resolveYamlVerificationPlan(options) {
  const {
    yamlPath,
    legacyCslPath = null,
    refsFixture = null,
    citationsFixture = null,
    fixtureFamily = null,
    styleName: explicitStyleName = null,
    styleData = null,
    styleFormat = null,
    hasBibliography = null,
  } = options;

  const styleName = explicitStyleName || path.basename(yamlPath, '.yaml');
  const verificationPolicy = loadVerificationPolicy();
  const fixtureSufficiency = loadFixtureSufficiency();
  const stylePolicy = resolveVerificationPolicy(styleName, verificationPolicy);
  const effectiveFixtureFamily = fixtureFamily || stylePolicy.fixtureFamily || null;
  const sufficiencyPolicy = effectiveFixtureFamily
    ? resolveFixtureSufficiency(effectiveFixtureFamily, fixtureSufficiency)
    : {
      family: null,
      defaultReportSufficient: true,
      requiredReferenceTypes: [],
      requiredScenarios: [],
      fixtureSets: [],
    };
  const sourceName = inferLegacySourceName(styleName, styleData);
  const resolvedLegacyCslPath = legacyCslPath
    ? path.resolve(legacyCslPath)
    : path.join(LEGACY_STYLES_DIR, `${sourceName}.csl`);
  const baseRun = {
    refsFixture: refsFixture ? path.resolve(refsFixture) : DEFAULT_REFS_FIXTURE,
    citationsFixture: citationsFixture
      ? path.resolve(citationsFixture)
      : resolveDefaultCitationFixture(styleFormat),
  };
  const familyRuns = (!refsFixture && !citationsFixture)
    ? getAdditionalFixtureSetNames(sufficiencyPolicy.fixtureSets)
      .map((setName) => ({ setName, ...getFixtureFiles(setName) }))
      .filter((entry) => entry.refsFixture && entry.citationsFixture)
    : [];

  return {
    styleName,
    sourceName,
    legacyCslPath: resolvedLegacyCslPath,
    stylePolicy,
    sufficiencyPolicy,
    effectiveScopes: getEffectiveVerificationScopes(stylePolicy, hasBibliography),
    baseRun,
    familyRuns,
    canUseStructuredOracle: isProjectStylePath(yamlPath),
  };
}

module.exports = {
  DEFAULT_CITATIONS_FIXTURE,
  DEFAULT_REFS_FIXTURE,
  FIXTURE_SET_CITATIONS,
  FIXTURE_SET_REFS,
  LEGACY_SOURCE_OVERRIDES,
  LEGACY_STYLES_DIR,
  NOTE_CITATIONS_FIXTURE,
  PROJECT_ROOT,
  STYLES_DIR,
  getAdditionalFixtureSetNames,
  getEffectiveVerificationScopes,
  getFixtureFiles,
  inferLegacySourceName,
  isProjectStylePath,
  resolveDefaultCitationFixture,
  resolveYamlVerificationPlan,
};
