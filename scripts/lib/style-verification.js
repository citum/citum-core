'use strict';

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');
const { execFileSync } = require('child_process');

const PROJECT_ROOT = path.resolve(__dirname, '..', '..');
const STYLES_DIR = path.join(PROJECT_ROOT, 'styles');
const LEGACY_STYLES_DIR = path.join(PROJECT_ROOT, 'styles-legacy');

const DEFAULT_REFS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json');
const DEFAULT_CITATIONS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-expanded.json');
const NOTE_CITATIONS_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-note-expanded.json');

const FIXTURE_SET_REFS = {
  'author-date': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json'),
  'physics-numeric': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json'),
  'humanities-note': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-humanities-note.json'),
  'secondary-roles': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-secondary-roles.json'),
};

const FIXTURE_SET_CITATIONS = {
  'author-date': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-expanded.json'),
  'physics-numeric': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-expanded.json'),
  'humanities-note': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-humanities-note.json'),
  'secondary-roles': path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-secondary-roles.json'),
};

const LEGACY_SOURCE_OVERRIDES = {
  'apa-7th': 'apa',
  'gost-r-7-0-5-2008-author-date': 'gost-r-7-0-5-2008',
  'chicago-notes-18th': 'chicago-notes',
  'chicago-author-date-18th': 'chicago-author-date',
  'taylor-and-francis-chicago-author-date': 'chicago-author-date',
};

const {
  loadVerificationPolicy,
  loadFixtureSufficiency,
  resolveVerificationPolicy,
  resolveFixtureSufficiency,
} = require('./verification-policy');

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
  const abs = path.resolve(yamlPath);
  const dir = path.dirname(abs);
  return dir === STYLES_DIR || dir === path.join(STYLES_DIR, 'embedded');
}

function resolveYamlVerificationPlan(options) {
  const {
    yamlPath,
    legacyCslPath = null,
    refsFixture = null,
    citationsFixture = null,
    fixtureFamily = null,
    styleName: explicitStyleName = null,
    styleData: rawStyleData = null,
    resolvedStyleData = null,
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

  const isProjectStyle = isProjectStylePath(yamlPath);
  const sourceName = isProjectStyle
    ? inferLegacySourceName(styleName, rawStyleData)
    : styleName;

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
    canUseStructuredOracle: isProjectStyle && fs.existsSync(resolvedLegacyCslPath),
  };
}

module.exports = {
  PROJECT_ROOT,
  STYLES_DIR,
  LEGACY_STYLES_DIR,
  DEFAULT_REFS_FIXTURE,
  DEFAULT_CITATIONS_FIXTURE,
  NOTE_CITATIONS_FIXTURE,
  FIXTURE_SET_REFS,
  FIXTURE_SET_CITATIONS,
  inferLegacySourceName,
  isProjectStylePath,
  resolveYamlVerificationPlan,
  getEffectiveVerificationScopes,
  getAdditionalFixtureSetNames,
  getFixtureFiles,
};
