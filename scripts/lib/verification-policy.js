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

function validateScopePolicy(scopePolicy, label) {
  assert(scopePolicy && typeof scopePolicy === 'object' && !Array.isArray(scopePolicy), `${label} must be an object`);
  if (scopePolicy.authority != null) {
    ensureAuthority(scopePolicy.authority, `${label}.authority`);
  }
  ensureOptionalString(scopePolicy.authority_id, `${label}.authority_id`);
  ensureOptionalString(scopePolicy.note, `${label}.note`);
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
  };
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
  DEFAULT_POLICY_PATH,
  DEFAULT_SUFFICIENCY_PATH,
  loadFixtureSufficiency,
  loadVerificationPolicy,
  resolveFixtureSufficiency,
  resolveVerificationPolicy,
  resolveScopeAuthority,
  validateFixtureSufficiency,
  validateVerificationPolicy,
};
