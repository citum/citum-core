#!/usr/bin/env node
/*
 * Audit fixture coverage for citum-migrate measured template selection.
 *
 * The selection fixture (`references-expanded.json`) is used while scoring
 * candidates. The held-out fixture (`references-heldout.json`) is used only
 * after selection to detect over-fitting. This script reports where those two
 * surfaces are strong, weak, or absent against the fixture manifest and the
 * family-level sufficiency policy.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const {
  loadFixtureSufficiency,
  DEFAULT_SUFFICIENCY_PATH,
} = require('./lib/verification-policy');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const SELECTION_FIXTURE = 'tests/fixtures/references-expanded.json';
const HELDOUT_FIXTURE = 'tests/fixtures/references-heldout.json';
const COVERAGE_MANIFEST = 'tests/fixtures/coverage-manifest.json';
const IMPORT_MANIFEST = 'tests/fixtures/measured-selection-imports.json';
const HIGH_RISK_TYPES = [
  'legal_case',
  'legislation',
  'patent',
  'dataset',
  'standard',
  'entry-encyclopedia',
  'entry-dictionary',
  'article-newspaper',
  'article-magazine',
  'broadcast',
  'interview',
  'map',
  'thesis',
  'report',
  'webpage',
];
const BEHAVIOR_FAMILIES = [
  'URL/accessed gating',
  'DOI suppression',
  'contributor role fallback',
  'editor/name-order handling',
  'title casing',
  'volume/pages delimiters',
  'issued/accessed/date-parts',
  'locale terms',
  'type-specific bibliography flattening',
  'first/subsequent/ibid citation position',
];

function usage() {
  console.error(`Usage:
  node scripts/audit-measured-fixture-coverage.js [--json] [--scorecard <path>]

Options:
  --json              Emit machine-readable JSON.
  --scorecard <path>  Include measured-selection summaries from a
                      report-migrate-sqi JSON file.`);
}

function parseArgs(argv) {
  const args = {
    json: false,
    scorecard: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--json') {
      args.json = true;
    } else if (arg === '--scorecard') {
      index += 1;
      if (!argv[index]) {
        throw new Error('--scorecard requires a path');
      }
      args.scorecard = argv[index];
    } else if (arg === '-h' || arg === '--help') {
      usage();
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return args;
}

function readJson(repoRelativePath) {
  return JSON.parse(fs.readFileSync(path.join(PROJECT_ROOT, repoRelativePath), 'utf8'));
}

function fixtureEntries(repoRelativePath) {
  const data = readJson(repoRelativePath);
  if (Array.isArray(data)) {
    return data;
  }
  return Object.entries(data)
    .filter(([id, value]) => id !== 'comment' && value && typeof value === 'object')
    .map(([id, value]) => ({ id, ...value }));
}

function countsByType(entries) {
  const counts = new Map();
  for (const entry of entries) {
    const type = entry.type || 'unknown';
    counts.set(type, (counts.get(type) || 0) + 1);
  }
  return Object.fromEntries([...counts.entries()].sort(([left], [right]) => left.localeCompare(right)));
}

function typeStatus(selectionCounts, heldoutCounts) {
  const types = new Set([
    ...Object.keys(selectionCounts),
    ...Object.keys(heldoutCounts),
    ...HIGH_RISK_TYPES,
  ]);
  return [...types].sort().map((type) => {
    const selection = selectionCounts[type] || 0;
    const heldout = heldoutCounts[type] || 0;
    let status = 'neither';
    if (selection > 0 && heldout > 0) status = 'both';
    else if (selection > 0) status = 'selection-only';
    else if (heldout > 0) status = 'heldout-only';
    return {
      type,
      selection,
      heldout,
      status,
      highRisk: HIGH_RISK_TYPES.includes(type),
    };
  });
}

function disjointness(selectionEntries, heldoutEntries) {
  const selectionIds = new Set(selectionEntries.map((entry) => entry.id));
  const selectionTitles = new Set(
    selectionEntries.map((entry) => String(entry.title || '').trim().toLowerCase()).filter(Boolean)
  );
  return {
    overlappingIds: heldoutEntries
      .filter((entry) => selectionIds.has(entry.id))
      .map((entry) => entry.id),
    overlappingTitles: heldoutEntries
      .filter((entry) => selectionTitles.has(String(entry.title || '').trim().toLowerCase()))
      .map((entry) => entry.id),
  };
}

function manifestEntryByFixture(manifest) {
  return new Map((manifest.fixtures || []).map((entry) => [entry.fixture, entry]));
}

function behaviorCoverage(manifest) {
  const entries = manifest.fixtures || [];
  const riskText = entries
    .flatMap((entry) => entry.rendering_risks || [])
    .join('\n')
    .toLowerCase();
  return BEHAVIOR_FAMILIES.map((family) => {
    const needles = family
      .toLowerCase()
      .split(/[ /-]+/)
      .filter((part) => part.length >= 4);
    const manifestMentioned = needles.some((part) => riskText.includes(part));
    return {
      family,
      manifestMentioned,
      note: manifestMentioned
        ? 'represented somewhere in coverage-manifest rendering risks'
        : 'not explicitly named in coverage-manifest rendering risks',
    };
  });
}

function familySufficiency(sufficiency, typeRows) {
  const coveredBySelection = new Set(typeRows.filter((row) => row.selection > 0).map((row) => row.type));
  const coveredByHeldout = new Set(typeRows.filter((row) => row.heldout > 0).map((row) => row.type));
  return Object.entries(sufficiency.families || {}).map(([family, config]) => {
    const requiredTypes = config.required_reference_types || [];
    return {
      family,
      defaultReportSufficient: Boolean(config.default_report_sufficient),
      fixtureSets: config.fixture_sets || [],
      missingSelectionTypes: requiredTypes.filter((type) => !coveredBySelection.has(type)),
      missingHeldoutTypes: requiredTypes.filter((type) => !coveredByHeldout.has(type)),
      requiredScenarios: config.required_scenarios || [],
    };
  });
}

function rankedDeferredGaps(typeRows, familyRows) {
  const gaps = [];
  for (const row of typeRows) {
    if (!row.highRisk) continue;
    if (row.selection === 0) {
      gaps.push({
        priority: 1,
        type: row.type,
        gap: 'selection-set',
        reason: 'high-risk reference type has no measured-selection item',
      });
    } else if (row.heldout === 0) {
      gaps.push({
        priority: 2,
        type: row.type,
        gap: 'held-out-set',
        reason: 'high-risk reference type can influence selection but cannot reject over-fitting',
      });
    }
  }
  for (const family of familyRows) {
    for (const type of family.missingSelectionTypes) {
      gaps.push({
        priority: 3,
        type,
        gap: 'selection-set',
        reason: `${family.family} sufficiency policy requires this type`,
      });
    }
    for (const type of family.missingHeldoutTypes) {
      gaps.push({
        priority: 4,
        type,
        gap: 'held-out-set',
        reason: `${family.family} sufficiency policy requires this type`,
      });
    }
  }
  const seen = new Set();
  return gaps
    .filter((gap) => {
      const key = `${gap.priority}:${gap.type}:${gap.gap}:${gap.reason}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    })
    .sort((left, right) => left.priority - right.priority || left.type.localeCompare(right.type));
}

function loadScorecard(scorecardPath) {
  if (!scorecardPath) return [];
  const absolute = path.resolve(scorecardPath);
  const data = JSON.parse(fs.readFileSync(absolute, 'utf8'));
  const rows = Array.isArray(data) ? data : data.styles || data.rows || data.results || [];
  return rows.flatMap((row) => {
    const evidence = row.evidence || {};
    const measured = evidence.measured_selection || {};
    const style = row.style || row.name || row.styleName || evidence.style_id || 'unknown';
    return ['citation', 'bibliography']
      .filter((section) => measured[section])
      .map((section) => ({
        style,
        section,
        selectedCandidate: measured[section].selected_candidate,
        useXml: Boolean(measured[section].use_xml),
        selectedPasses: measured[section].selected_passes,
        inferredPasses: measured[section].inferred_passes,
        xmlPasses: measured[section].xml_passes,
        items: measured[section].items,
        heldout: measured[section].heldout || null,
      }));
  });
}

function buildReport(options) {
  const selectionEntries = fixtureEntries(SELECTION_FIXTURE);
  const heldoutEntries = fixtureEntries(HELDOUT_FIXTURE);
  const selectionCounts = countsByType(selectionEntries);
  const heldoutCounts = countsByType(heldoutEntries);
  const manifest = readJson(COVERAGE_MANIFEST);
  const manifestByFixture = manifestEntryByFixture(manifest);
  const sufficiency = loadFixtureSufficiency(DEFAULT_SUFFICIENCY_PATH);
  const types = typeStatus(selectionCounts, heldoutCounts);
  const families = familySufficiency(sufficiency, types);
  return {
    generatedBy: 'scripts/audit-measured-fixture-coverage.js',
    inputs: {
      selectionFixture: SELECTION_FIXTURE,
      heldoutFixture: HELDOUT_FIXTURE,
      coverageManifest: COVERAGE_MANIFEST,
      importManifest: IMPORT_MANIFEST,
      fixtureSufficiency: path.relative(PROJECT_ROOT, DEFAULT_SUFFICIENCY_PATH),
      scorecard: options.scorecard || null,
    },
    fixtureSufficiencyProvenance: {
      status: 'checked-in policy file',
      validatedBy: 'scripts/check-testing-infra.js',
      note: 'This file is hand-maintained policy metadata for fixture-family sufficiency; it is not generated by a script in this repository.',
    },
    summary: {
      selectionItems: selectionEntries.length,
      selectionTypes: Object.keys(selectionCounts).length,
      heldoutItems: heldoutEntries.length,
      heldoutTypes: Object.keys(heldoutCounts).length,
      disjoint: disjointness(selectionEntries, heldoutEntries),
    },
    manifest: {
      selection: manifestByFixture.get(SELECTION_FIXTURE) || null,
      heldout: manifestByFixture.get(HELDOUT_FIXTURE) || null,
    },
    referenceTypes: types,
    behaviorFamilies: behaviorCoverage(manifest),
    fixtureFamilies: families,
    deferredGaps: rankedDeferredGaps(types, families),
    measuredSelection: loadScorecard(options.scorecard),
  };
}

function table(headers, rows) {
  const divider = headers.map(() => '---');
  const lines = [headers, divider, ...rows].map((row) => `| ${row.join(' | ')} |`);
  return lines.join('\n');
}

function renderMarkdown(report) {
  const typeRows = report.referenceTypes.map((row) => [
    `\`${row.type}\``,
    String(row.selection),
    String(row.heldout),
    row.status,
    row.highRisk ? 'yes' : 'no',
  ]);
  const familyRows = report.fixtureFamilies.map((row) => [
    row.family,
    row.defaultReportSufficient ? 'yes' : 'no',
    row.missingSelectionTypes.map((type) => `\`${type}\``).join(', ') || 'none',
    row.missingHeldoutTypes.map((type) => `\`${type}\``).join(', ') || 'none',
  ]);
  const gapRows = report.deferredGaps.slice(0, 25).map((gap) => [
    String(gap.priority),
    `\`${gap.type}\``,
    gap.gap,
    gap.reason,
  ]);
  const selectionRows = report.measuredSelection.map((row) => [
    row.style,
    row.section,
    row.selectedCandidate,
    row.useXml ? 'yes' : 'no',
    `${row.selectedPasses}/${row.items}`,
    row.heldout ? `${row.heldout.passes}/${row.heldout.items}` : 'n/a',
  ]);

  return `# Measured Selection Fixture Coverage Audit

Generated by \`${report.generatedBy}\`.

## Inputs

- Selection fixture: \`${report.inputs.selectionFixture}\`
- Held-out fixture: \`${report.inputs.heldoutFixture}\`
- Coverage manifest: \`${report.inputs.coverageManifest}\`
- Curated import manifest: \`${report.inputs.importManifest}\`
- Fixture sufficiency policy: \`${report.inputs.fixtureSufficiency}\`

\`${report.inputs.fixtureSufficiency}\` is a checked-in policy file, not a generated
artifact. It is validated by \`scripts/check-testing-infra.js\` and records the
reference types, scenarios, and fixture sets considered sufficient for each
verification family.

\`${report.inputs.importManifest}\` records the allowlisted CSL JSON source
items imported from \`tests/fixtures/test-items-library/\`. The derived
\`TLIB-*\` entries stay CSL JSON at this CSL/Citum boundary; type overrides
only materialize Zotero Extra \`type:\` hints as explicit top-level CSL item
types so citeproc-js and Citum score the same effective item.

## Summary

- Selection set: ${report.summary.selectionItems} items across ${report.summary.selectionTypes} reference types.
- Held-out set: ${report.summary.heldoutItems} items across ${report.summary.heldoutTypes} reference types.
- Disjointness: ${report.summary.disjoint.overlappingIds.length === 0 && report.summary.disjoint.overlappingTitles.length === 0 ? 'no ID or title overlap detected' : 'overlap detected'}.

## Reference Type Surface

${table(['Reference type', 'Selection', 'Held-out', 'Status', 'High risk'], typeRows)}

## Fixture-Family Sufficiency

${table(['Family', 'Default sufficient', 'Missing selection types', 'Missing held-out types'], familyRows)}

## Behavior Families

${table(
  ['Behavior family', 'Manifest signal', 'Note'],
  report.behaviorFamilies.map((row) => [
    row.family,
    row.manifestMentioned ? 'yes' : 'no',
    row.note,
  ])
)}

## Ranked Deferred Fixture Gaps

${gapRows.length > 0 ? table(['Priority', 'Type', 'Gap', 'Reason'], gapRows) : 'No deferred gaps found.'}

## Measured-Selection Evidence

${selectionRows.length > 0 ? table(['Style', 'Section', 'Selected candidate', 'XML', 'Selection passes', 'Held-out passes'], selectionRows) : 'No scorecard JSON was provided. Pass `--scorecard <path>` after running `scripts/report-migrate-sqi.js` to include measured-selection winners.'}
`;
}

function main() {
  try {
    const options = parseArgs(process.argv.slice(2));
    const report = buildReport(options);
    if (options.json) {
      process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    } else {
      process.stdout.write(renderMarkdown(report));
    }
  } catch (error) {
    console.error(`audit-measured-fixture-coverage failed: ${error.message}`);
    usage();
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  buildReport,
  renderMarkdown,
};
