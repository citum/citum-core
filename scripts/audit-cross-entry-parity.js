#!/usr/bin/env node
/**
 * Cross-Entry Parity Audit
 *
 * For each style in styles/ (and styles/experimental/), resolves its
 * source CSL from styles-legacy/ and checks whether:
 *
 *  1. subsequent-author-substitute in the CSL is reflected in the YAML
 *  2. disambiguate-* flags in the CSL match the YAML's processing.disambiguate
 *     (only flagged when YAML uses processing: custom or explicitly overrides)
 *
 * A style that uses `processing: author-date` inherits all disambiguation
 * flags from the Processing::AuthorDate default — no explicit YAML field
 * is required. Only `subsequent-author-substitute` must be explicit.
 *
 * Usage:
 *   node audit-cross-entry-parity.js              # human-readable
 *   node audit-cross-entry-parity.js --json        # machine-readable JSON
 *   node audit-cross-entry-parity.js --fix-report  # same as --json but named for CI
 *
 * Exit codes:
 *   0  All checked styles pass
 *   1  One or more offenders found (or --strict mode)
 *   2  Fatal error
 */

'use strict';

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const STYLES_DIRS = [
  path.join(PROJECT_ROOT, 'styles'),
  path.join(PROJECT_ROOT, 'styles', 'experimental'),
];
const LEGACY_DIR = path.join(PROJECT_ROOT, 'styles-legacy');

// Styles that legitimately omit subsequent-author-substitute even when
// their source CSL has it (document the reason in the comment).
const KNOWN_ALLOWLIST = new Set([
  // Add style IDs here if they deliberately diverge from CSL source.
]);

// Native Citum styles that should have subsequent-author-substitute but
// cannot be verified against a source CSL (the Zotero CSL omits the feature,
// or the style is authored from scratch against the official style guide).
// Format: styleId → { substitute, rule }
const NATIVE_STYLE_EXPECTATIONS = new Map([
  [
    'chicago-author-date',
    {
      substitute: '———',
      rule: 'complete-all',
      reason:
        'CMOS 18th ed. (§14.67) uses 3-em-dash for repeated author groups; ' +
        'Zotero chicago-author-date.csl omits this feature.',
    },
  ],
]);

// ─── CSL parsing helpers ──────────────────────────────────────────────────────

function parseCslBibliographyAttrs(cslContent) {
  const result = {
    subsequentAuthorSubstitute: null,
    subsequentAuthorSubstituteRule: null,
    disambiguateAddNames: false,
    disambiguateAddGivenname: false,
    disambiguateAddYearSuffix: false,
    givennameDisambiguationRule: null,
  };

  // subsequent-author-substitute (attribute on <bibliography ...>)
  const subMatch = cslContent.match(
    /subsequent-author-substitute="([^"]*)"/
  );
  if (subMatch) result.subsequentAuthorSubstitute = subMatch[1];

  const subRuleMatch = cslContent.match(
    /subsequent-author-substitute-rule="([^"]*)"/
  );
  if (subRuleMatch) result.subsequentAuthorSubstituteRule = subRuleMatch[1];

  // disambiguate flags (attributes on <citation> and checked globally)
  if (/disambiguate-add-names="true"/.test(cslContent))
    result.disambiguateAddNames = true;
  if (/disambiguate-add-givenname="true"/.test(cslContent))
    result.disambiguateAddGivenname = true;
  if (/disambiguate-add-year-suffix="true"/.test(cslContent))
    result.disambiguateAddYearSuffix = true;

  const gnRuleMatch = cslContent.match(
    /givenname-disambiguation-rule="([^"]*)"/
  );
  if (gnRuleMatch) result.givennameDisambiguationRule = gnRuleMatch[1];

  return result;
}

// ─── YAML analysis helpers ────────────────────────────────────────────────────

function yamlSubsequentSubstitute(style) {
  return (
    style?.bibliography?.options?.['subsequent-author-substitute'] ??
    style?.bibliography?.options?.subsequent_author_substitute ??
    null
  );
}

function yamlSubsequentSubstituteRule(style) {
  return (
    style?.bibliography?.options?.['subsequent-author-substitute-rule'] ??
    style?.bibliography?.options?.subsequent_author_substitute_rule ??
    null
  );
}

/** Normalize entity-encoded or ASCII dashes to literal Unicode dashes for comparison. */
function normalizeDash(str) {
  if (!str) return str;
  return str
    .replace(/&#8212;/g, '—')
    .replace(/&#8211;/g, '–')
    .replace(/---/g, '———')
    .replace(/--/g, '——');
}

function yamlProcessingMode(style) {
  const proc = style?.options?.processing;
  if (!proc) return 'author-date'; // default
  if (typeof proc === 'string') return proc;
  if (typeof proc === 'object') {
    if ('label' in proc) return 'label';
    return 'custom';
  }
  return 'unknown';
}

function yamlDisambiguate(style) {
  const proc = style?.options?.processing;
  if (typeof proc !== 'object' || !proc) return null;
  return proc.disambiguate ?? null;
}

// ─── Style resolution ─────────────────────────────────────────────────────────

function resolveSourceCsl(styleId) {
  // Direct match
  const direct = path.join(LEGACY_DIR, `${styleId}.csl`);
  if (fs.existsSync(direct)) return direct;
  return null;
}

function extractStyleId(stylePath) {
  // Try info.id from YAML
  try {
    const content = fs.readFileSync(stylePath, 'utf8');
    const style = yaml.load(content);
    if (style?.info?.id) {
      // Extract just the last path segment, no .csl extension
      return style.info.id.split('/').pop().replace(/\.csl$/, '');
    }
  } catch (_) {}
  // Fall back to filename without extension
  return path.basename(stylePath, '.yaml');
}

// ─── Audit logic ──────────────────────────────────────────────────────────────

function auditStyle(stylePath) {
  const styleId = extractStyleId(stylePath);
  const result = {
    styleId,
    stylePath: path.relative(PROJECT_ROOT, stylePath),
    cslPath: null,
    issues: [],
    skipped: false,
    skipReason: null,
  };

  if (KNOWN_ALLOWLIST.has(styleId)) {
    result.skipped = true;
    result.skipReason = 'in known allowlist';
    return result;
  }

  let style;
  try {
    style = yaml.load(fs.readFileSync(stylePath, 'utf8'));
  } catch (err) {
    result.issues.push(`Failed to parse YAML: ${err.message}`);
    return result;
  }

  // Check native expectations first — these take precedence over the source CSL
  // for styles where the Zotero/community CSL deliberately omits a feature that
  // the official style guide requires.
  const nativeExpectation = NATIVE_STYLE_EXPECTATIONS.get(styleId);
  if (nativeExpectation) {
    const yamlSub = yamlSubsequentSubstitute(style);
    const yamlRule = yamlSubsequentSubstituteRule(style);
    if (yamlSub === null) {
      result.issues.push(
        `subsequent-author-substitute missing (expected "${nativeExpectation.substitute}") — ` +
          nativeExpectation.reason
      );
    } else if (normalizeDash(yamlSub) !== normalizeDash(nativeExpectation.substitute)) {
      result.issues.push(
        `subsequent-author-substitute value mismatch: ` +
          `YAML="${yamlSub}" vs expected "${nativeExpectation.substitute}"`
      );
    }
    if (nativeExpectation.rule) {
      if (!yamlRule) {
        result.issues.push(
          `subsequent-author-substitute-rule missing (expected "${nativeExpectation.rule}")`
        );
      } else if (yamlRule !== nativeExpectation.rule) {
        result.issues.push(
          `subsequent-author-substitute-rule mismatch: ` +
            `YAML="${yamlRule}" vs expected "${nativeExpectation.rule}"`
        );
      }
    }
    // Native expectations fully define what's required; skip further CSL checks.
    return result;
  }

  const cslPath = resolveSourceCsl(styleId);
  if (!cslPath) {
    result.skipped = true;
    result.skipReason = 'no source CSL found (native style)';
    return result;
  }

  result.cslPath = path.relative(PROJECT_ROOT, cslPath);

  let cslContent;
  try {
    cslContent = fs.readFileSync(cslPath, 'utf8');
  } catch (err) {
    result.issues.push(`Failed to read CSL: ${err.message}`);
    return result;
  }

  const cslAttrs = parseCslBibliographyAttrs(cslContent);
  const processingMode = yamlProcessingMode(style);

  // ── Check 1: subsequent-author-substitute ──────────────────────────────────
  if (cslAttrs.subsequentAuthorSubstitute !== null) {
    const yamlSub = yamlSubsequentSubstitute(style);
    if (yamlSub === null) {
      result.issues.push(
        `subsequent-author-substitute missing in YAML ` +
          `(CSL has: "${cslAttrs.subsequentAuthorSubstitute}")`
      );
    } else if (normalizeDash(cslAttrs.subsequentAuthorSubstitute) !== normalizeDash(yamlSub)) {
      // Flag value drift (entity vs literal forms are normalized before comparison)
      result.issues.push(
        `subsequent-author-substitute value mismatch: ` +
          `YAML="${yamlSub}" vs CSL="${cslAttrs.subsequentAuthorSubstitute}"`
      );
    }
    // Compare rule when CSL declares one
    if (cslAttrs.subsequentAuthorSubstituteRule !== null) {
      const yamlRule = yamlSubsequentSubstituteRule(style);
      const cslRule = cslAttrs.subsequentAuthorSubstituteRule;
      if (!yamlRule) {
        result.issues.push(
          `subsequent-author-substitute-rule missing in YAML (CSL has: "${cslRule}")`
        );
      } else if (yamlRule !== cslRule) {
        result.issues.push(
          `subsequent-author-substitute-rule mismatch: YAML="${yamlRule}" vs CSL="${cslRule}"`
        );
      }
    }
  }

  // ── Check 2: disambiguation config ────────────────────────────────────────
  // Only flag when the YAML uses `processing: custom` and omits disambiguate,
  // but the CSL source declares disambiguate flags.
  if (
    processingMode === 'custom' &&
    (cslAttrs.disambiguateAddNames ||
      cslAttrs.disambiguateAddGivenname ||
      cslAttrs.disambiguateAddYearSuffix)
  ) {
    const yamlDisamb = yamlDisambiguate(style);
    if (!yamlDisamb) {
      result.issues.push(
        `processing: custom but disambiguate missing in YAML ` +
          `(CSL flags: add-names=${cslAttrs.disambiguateAddNames}, ` +
          `add-givenname=${cslAttrs.disambiguateAddGivenname}, ` +
          `year-suffix=${cslAttrs.disambiguateAddYearSuffix})`
      );
    }
  }

  return result;
}

// ─── Main ─────────────────────────────────────────────────────────────────────

function main() {
  const args = process.argv.slice(2);
  const jsonMode = args.includes('--json') || args.includes('--fix-report');

  const allResults = [];

  for (const dir of STYLES_DIRS) {
    if (!fs.existsSync(dir)) continue;
    const files = fs.readdirSync(dir).filter((f) => f.endsWith('.yaml'));
    for (const file of files) {
      const stylePath = path.join(dir, file);
      allResults.push(auditStyle(stylePath));
    }
  }

  const offenders = allResults.filter(
    (r) => !r.skipped && r.issues.length > 0
  );
  const checked = allResults.filter((r) => !r.skipped);
  const skipped = allResults.filter((r) => r.skipped);

  if (jsonMode) {
    process.stdout.write(
      JSON.stringify(
        {
          summary: {
            total: allResults.length,
            checked: checked.length,
            skipped: skipped.length,
            offenders: offenders.length,
          },
          offenders,
          skipped: skipped.map((r) => ({
            styleId: r.styleId,
            reason: r.skipReason,
          })),
        },
        null,
        2
      )
    );
    process.stdout.write('\n');
  } else {
    console.log(
      `\n=== Cross-Entry Parity Audit: ${checked.length} checked, ${skipped.length} skipped ===\n`
    );

    if (offenders.length === 0) {
      console.log('✅ No offenders found.');
    } else {
      console.log(`❌ ${offenders.length} offender(s):\n`);
      for (const r of offenders) {
        console.log(`  ${r.styleId} (${r.stylePath})`);
        if (r.cslPath) console.log(`    CSL: ${r.cslPath}`);
        for (const issue of r.issues) {
          console.log(`    ⚠  ${issue}`);
        }
      }
    }

    console.log(
      `\nTotal: ${allResults.length} styles, ${offenders.length} offender(s), ${skipped.length} skipped (no source CSL or allowlisted)\n`
    );
  }

  process.exit(offenders.length > 0 ? 1 : 0);
}

main();
