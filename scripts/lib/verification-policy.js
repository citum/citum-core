'use strict';

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');

const PROJECT_ROOT = path.resolve(__dirname, '..', '..');
const DEFAULT_POLICY_PATH = path.join(PROJECT_ROOT, 'scripts', 'report-data', 'verification-policy.yaml');
const DEFAULT_SUFFICIENCY_PATH = path.join(PROJECT_ROOT, 'scripts', 'report-data', 'fixture-sufficiency.yaml');

const ALLOWED_AUTHORITIES = new Set([
  'citeproc-js',
  'biblatex',
  'citum-baseline',
  'documentary',
]);
const ALLOWED_BENCHMARK_RUNNERS = new Set([
  'citeproc-oracle',
  'native-smoke',
]);
const ALLOWED_BENCHMARK_SCOPES = new Set([
  'citation',
  'bibliography',
  'both',
]);

function readYaml(filePath) {
  return yaml.load(fs.readFileSync(filePath, 'utf8')) || {};
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function ensureAuthority(value, label) {
  assert(typeof value === 'string' && ALLOWED_AUTHORITIES.has(value), `${label} must be one of: ${[...ALLOWED_AUTHORITIES].join(', ')}`);
}

function ensureStringArray(value, label) {
  assert(Array.isArray(value), `${label} must be an array`);
  for (const entry of value) {
    assert(typeof entry === 'string' && entry.trim().length > 0, `${label} entries must be non-empty strings`);
  }
}

function ensureOptionalString(value, label) {
  if (value == null) return;
  assert(typeof value === 'string' && value.trim().length > 0, `${label} must be a non-empty string`);
}

function validateRegisteredDivergence(divergence, label) {
  assert(divergence && typeof divergence === 'object' && !Array.isArray(divergence), `${label} must be an object`);
  ensureStringArray(divergence.scopes || [], `${label}.scopes`);
  ensureStringArray(divergence.tags || [], `${label}.tags`);
  ensureOptionalString(divergence.note, `${label}.note`);
}

function validateScopePolicy(scopePolicy, label) {
  assert(scopePolicy && typeof scopePolicy === 'object' && !Array.isArray(scopePolicy), `${label} must be an object`);
  if (scopePolicy.authority != null) {
    ensureAuthority(scopePolicy.authority, `${label}.authority`);
  }
  ensureOptionalString(scopePolicy.authority_id, `${label}.authority_id`);
  ensureOptionalString(scopePolicy.note, `${label}.note`);
}

function validateBenchmarkRun(run, label) {
  assert(run && typeof run === 'object' && !Array.isArray(run), `${label} must be an object`);
  ensureOptionalString(run.id, `${label}.id`);
  ensureOptionalString(run.label, `${label}.label`);
  assert(
    typeof run.runner === 'string' && ALLOWED_BENCHMARK_RUNNERS.has(run.runner),
    `${label}.runner must be one of: ${[...ALLOWED_BENCHMARK_RUNNERS].join(', ')}`
  );
  assert(
    typeof run.scope === 'string' && ALLOWED_BENCHMARK_SCOPES.has(run.scope),
    `${label}.scope must be one of: ${[...ALLOWED_BENCHMARK_SCOPES].join(', ')}`
  );
  ensureOptionalString(run.refs_fixture, `${label}.refs_fixture`);
  ensureOptionalString(run.citations_fixture, `${label}.citations_fixture`);
  assert(typeof run.count_toward_fidelity === 'boolean', `${label}.count_toward_fidelity must be a boolean`);
  assert(run.id && run.label && run.refs_fixture, `${label} must define id, label, and refs_fixture`);
  if (run.runner === 'native-smoke') {
    assert(run.scope === 'bibliography', `${label}.scope must be bibliography for native-smoke runs`);
    assert(run.count_toward_fidelity === false, `${label}.count_toward_fidelity must be false for native-smoke runs`);
  }
  if (run.runner === 'citeproc-oracle') {
    assert(run.scope !== 'citation', `${label}.scope citation is not yet supported for citeproc-oracle runs`);
  }
  if (run.scope !== 'bibliography') {
    assert(run.citations_fixture, `${label}.citations_fixture is required unless scope is bibliography`);
  }
}

function validateVerificationPolicy(policy) {
  assert(policy && typeof policy === 'object' && !Array.isArray(policy), 'verification-policy.yaml must be an object');
  assert(policy.version === 1, 'verification-policy.yaml version must be 1');
  assert(policy.defaults && typeof policy.defaults === 'object', 'verification-policy.yaml must define defaults');
  assert(policy.styles && typeof policy.styles === 'object' && !Array.isArray(policy.styles), 'verification-policy.yaml must define styles');

  const defaults = policy.defaults;
  ensureAuthority(defaults.authority, 'verification-policy.yaml defaults.authority');
  ensureStringArray(defaults.secondary || [], 'verification-policy.yaml defaults.secondary');
  for (const authority of defaults.secondary || []) {
    ensureAuthority(authority, 'verification-policy.yaml defaults.secondary');
  }
  ensureStringArray(defaults.scopes || [], 'verification-policy.yaml defaults.scopes');
  if (policy.divergences != null) {
    assert(
      policy.divergences && typeof policy.divergences === 'object' && !Array.isArray(policy.divergences),
      'verification-policy.yaml divergences must be an object'
    );
    for (const [divergenceId, divergence] of Object.entries(policy.divergences)) {
      validateRegisteredDivergence(divergence, `verification-policy.yaml divergences.${divergenceId}`);
    }
  }

  for (const [styleName, stylePolicy] of Object.entries(policy.styles)) {
    assert(stylePolicy && typeof stylePolicy === 'object' && !Array.isArray(stylePolicy), `verification-policy.yaml styles.${styleName} must be an object`);
    if (stylePolicy.authority != null) {
      ensureAuthority(stylePolicy.authority, `verification-policy.yaml styles.${styleName}.authority`);
    }
    ensureOptionalString(stylePolicy.authority_id, `verification-policy.yaml styles.${styleName}.authority_id`);
    if (stylePolicy.secondary != null) {
      ensureStringArray(stylePolicy.secondary, `verification-policy.yaml styles.${styleName}.secondary`);
      for (const authority of stylePolicy.secondary) {
        ensureAuthority(authority, `verification-policy.yaml styles.${styleName}.secondary`);
      }
    }
    if (stylePolicy.scopes != null) {
      ensureStringArray(stylePolicy.scopes, `verification-policy.yaml styles.${styleName}.scopes`);
    }
    if (stylePolicy.fixture_family != null) {
      assert(typeof stylePolicy.fixture_family === 'string' && stylePolicy.fixture_family.trim().length > 0, `verification-policy.yaml styles.${styleName}.fixture_family must be a non-empty string`);
    }
    if (stylePolicy.note != null) {
      assert(typeof stylePolicy.note === 'string' && stylePolicy.note.trim().length > 0, `verification-policy.yaml styles.${styleName}.note must be a non-empty string`);
    }
    if (stylePolicy.regression_baseline != null) {
      ensureAuthority(stylePolicy.regression_baseline, `verification-policy.yaml styles.${styleName}.regression_baseline`);
    }
    if (stylePolicy.scope_authorities != null) {
      assert(
        stylePolicy.scope_authorities && typeof stylePolicy.scope_authorities === 'object' && !Array.isArray(stylePolicy.scope_authorities),
        `verification-policy.yaml styles.${styleName}.scope_authorities must be an object`
      );
      for (const [scopeName, scopePolicy] of Object.entries(stylePolicy.scope_authorities)) {
        validateScopePolicy(scopePolicy, `verification-policy.yaml styles.${styleName}.scope_authorities.${scopeName}`);
      }
    }
    if (stylePolicy.benchmark_runs != null) {
      assert(Array.isArray(stylePolicy.benchmark_runs), `verification-policy.yaml styles.${styleName}.benchmark_runs must be an array`);
      for (let index = 0; index < stylePolicy.benchmark_runs.length; index += 1) {
        validateBenchmarkRun(
          stylePolicy.benchmark_runs[index],
          `verification-policy.yaml styles.${styleName}.benchmark_runs[${index}]`
        );
      }
    }
  }

  return policy;
}

function validateFixtureSufficiency(config) {
  assert(config && typeof config === 'object' && !Array.isArray(config), 'fixture-sufficiency.yaml must be an object');
  assert(config.version === 1, 'fixture-sufficiency.yaml version must be 1');
  assert(config.defaults && typeof config.defaults === 'object', 'fixture-sufficiency.yaml must define defaults');
  assert(config.families && typeof config.families === 'object' && !Array.isArray(config.families), 'fixture-sufficiency.yaml must define families');

  for (const [familyName, family] of Object.entries(config.families)) {
    assert(family && typeof family === 'object' && !Array.isArray(family), `fixture-sufficiency.yaml families.${familyName} must be an object`);
    ensureStringArray(family.required_reference_types || [], `fixture-sufficiency.yaml families.${familyName}.required_reference_types`);
    ensureStringArray(family.required_scenarios || [], `fixture-sufficiency.yaml families.${familyName}.required_scenarios`);
    ensureStringArray(family.fixture_sets || [], `fixture-sufficiency.yaml families.${familyName}.fixture_sets`);
    assert(typeof family.default_report_sufficient === 'boolean', `fixture-sufficiency.yaml families.${familyName}.default_report_sufficient must be a boolean`);
  }

  return config;
}

function loadVerificationPolicy(policyPath = DEFAULT_POLICY_PATH) {
  return validateVerificationPolicy(readYaml(policyPath));
}

function loadFixtureSufficiency(configPath = DEFAULT_SUFFICIENCY_PATH) {
  return validateFixtureSufficiency(readYaml(configPath));
}

function resolveVerificationPolicy(styleName, policy) {
  const defaults = policy.defaults || {};
  const stylePolicy = policy.styles?.[styleName] || {};
  return {
    authority: stylePolicy.authority || defaults.authority,
    authorityId: stylePolicy.authority_id || null,
    secondary: stylePolicy.secondary || defaults.secondary || [],
    scopes: stylePolicy.scopes || defaults.scopes || [],
    fixtureFamily: stylePolicy.fixture_family || null,
    note: stylePolicy.note || null,
    regressionBaseline: stylePolicy.regression_baseline || null,
    scopeAuthorities: stylePolicy.scope_authorities || {},
    benchmarkRuns: (stylePolicy.benchmark_runs || []).map((run) => ({
      id: run.id,
      label: run.label,
      runner: run.runner,
      refsFixture: run.refs_fixture,
      citationsFixture: run.citations_fixture || null,
      scope: run.scope,
      countTowardFidelity: run.count_toward_fidelity,
    })),
  };
}

function resolveRegisteredDivergence(policy, divergenceId) {
  return policy?.divergences?.[divergenceId] || null;
}

function resolveScopeAuthority(policy, scopeName) {
  const scopePolicy = policy.scopeAuthorities?.[scopeName] || {};
  const hasScopeAuthority = Object.prototype.hasOwnProperty.call(scopePolicy, 'authority')
    || Object.prototype.hasOwnProperty.call(scopePolicy, 'authority_id');
  return {
    authority: scopePolicy.authority || policy.authority,
    authorityId: hasScopeAuthority
      ? (scopePolicy.authority_id || null)
      : policy.authorityId,
    note: scopePolicy.note || policy.note || null,
  };
}

function resolveFixtureSufficiency(familyName, config) {
  if (!familyName) {
    return {
      family: null,
      defaultReportSufficient: true,
      requiredReferenceTypes: [],
      requiredScenarios: [],
      fixtureSets: [],
    };
  }
  const family = config.families?.[familyName];
  assert(family, `fixture-sufficiency.yaml is missing family: ${familyName}`);
  return {
    family: familyName,
    defaultReportSufficient: family.default_report_sufficient,
    requiredReferenceTypes: family.required_reference_types || [],
    requiredScenarios: family.required_scenarios || [],
    fixtureSets: family.fixture_sets || [],
  };
}

module.exports = {
  ALLOWED_AUTHORITIES,
  ALLOWED_BENCHMARK_RUNNERS,
  ALLOWED_BENCHMARK_SCOPES,
  DEFAULT_POLICY_PATH,
  DEFAULT_SUFFICIENCY_PATH,
  loadFixtureSufficiency,
  loadVerificationPolicy,
  resolveFixtureSufficiency,
  resolveRegisteredDivergence,
  resolveVerificationPolicy,
  resolveScopeAuthority,
  validateFixtureSufficiency,
  validateVerificationPolicy,
  deepMerge,
  resolveStyleData,
};

/**
 * Perform a deep merge of objects for style variant resolution.
 */
function deepMerge(target, source) {
  if (!source || typeof source !== 'object') return target;
  if (!target || typeof target !== 'object') return source;

  const result = { ...target };
  for (const [key, value] of Object.entries(source)) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      result[key] = deepMerge(result[key], value);
    } else {
      result[key] = value;
    }
  }
  return result;
}

const PRESET_BASES = {
  'apa-7th': path.join(PROJECT_ROOT, 'styles', 'preset-bases', 'apa-7th.yaml'),
  'chicago-notes-18th': path.join(PROJECT_ROOT, 'styles', 'preset-bases', 'chicago-notes-18th.yaml'),
  'chicago-author-date-18th': path.join(PROJECT_ROOT, 'styles', 'preset-bases', 'chicago-author-date-18th.yaml'),
};

function localStyleOverlay(styleData) {
  if (!styleData || typeof styleData !== 'object') {
    return {};
  }

  const overlay = { ...styleData };
  delete overlay.preset;
  return overlay;
}

/**
 * Resolves a style's preset reference.
 */
function resolveStyleData(styleData, visited = new Set()) {
  const presetSpec = styleData?.preset;
  if (!presetSpec) return styleData;

  const presetKey = typeof presetSpec === 'string' ? presetSpec : presetSpec.preset;
  if (!presetKey || !PRESET_BASES[presetKey] || visited.has(presetKey)) {
    return styleData;
  }

  const basePath = PRESET_BASES[presetKey];
  if (!fs.existsSync(basePath)) {
    return styleData;
  }

  try {
    const baseContent = fs.readFileSync(basePath, 'utf8');
    let baseData = yaml.load(baseContent);

    visited.add(presetKey);
    baseData = resolveStyleData(baseData, visited);

    const delta = typeof presetSpec === 'object' ? presetSpec.variant : null;
    const mergedPreset = deepMerge(baseData, delta || {});
    return deepMerge(mergedPreset, localStyleOverlay(styleData));
  } catch (err) {
    console.error(`Error resolving preset ${presetKey}: ${err.message}`);
    return styleData;
  }
}
