#!/usr/bin/env node
/**
 * Generate a conservative Citum YAML scaffold from a biblatex snapshot.
 *
 * This is intentionally not a biblatex converter. It consumes rendered
 * bibliography output and emits a starter style that an author finishes by
 * hand.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const yaml = require('js-yaml');

const {
  analyzeOrdering,
  detectDelimiters,
  normalizeText,
  parseComponents,
} = require('./lib/component-parser');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const DEFAULT_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json');
const DEFAULT_SNAPSHOT_DIR = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'biblatex');
const SNAPSHOT_GENERATOR = path.join(PROJECT_ROOT, 'scripts', 'gen-biblatex-snapshot.js');

const NUMERIC_LABEL_COMPONENT = Object.freeze({
  number: 'citation-number',
  wrap: { punctuation: 'brackets' },
  suffix: ' ',
});

const COMPONENT_MAP = Object.freeze({
  contributors: { contributor: 'author', form: 'long' },
  title: { title: 'primary' },
  containerTitle: { title: 'parent-serial' },
  editors: { contributor: 'editor', form: 'verb' },
  publisher: { variable: 'publisher' },
  place: { variable: 'publisher-place' },
  volume: { number: 'volume' },
  issue: { number: 'issue' },
  year: { date: 'issued', form: 'year' },
  pages: { number: 'pages' },
  doi: { variable: 'doi' },
  url: { variable: 'url' },
  edition: { number: 'edition' },
});

function printUsage(stream = process.stderr) {
  stream.write(`Usage:
  node scripts/scaffold-biblatex-style.js --style <biblatex-style> [options]

Options:
  --style <name>           biblatex style name (required)
  --citum-style <name>     Citum style/snapshot stem (default: same as --style)
  --snapshot <path>        Snapshot JSON path (default: tests/snapshots/biblatex/<citum-style>.json)
  --fixture <path>         CSL JSON fixture (default: tests/fixtures/references-expanded.json)
  --output <path|->        Write YAML to path, or stdout with - (default: stdout)
  --title <text>           Human-readable title (default: title-cased --citum-style)
  --generate-snapshot      Run scripts/gen-biblatex-snapshot.js when snapshot is missing
  --force-snapshot         Regenerate snapshot even if it already exists
  --bib <path>             Forwarded to gen-biblatex-snapshot.js
  --biblatex-opts <opts>   Forwarded to gen-biblatex-snapshot.js
  --cite <id,id,...>       Fixture/snapshot key order; forwarded during generation
  -h, --help               Show this help

Output is a hand-finish scaffold, not a biblatex converter.
`);
}

function parseArgs(argv = process.argv.slice(2)) {
  const opts = {
    style: null,
    citumStyle: null,
    snapshot: null,
    snapshotExplicit: false,
    fixture: DEFAULT_FIXTURE,
    output: '-',
    title: null,
    generateSnapshot: false,
    forceSnapshot: false,
    bib: null,
    biblatexOpts: null,
    cite: null,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const nextValue = () => {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) {
        throw new Error(`${arg} requires a value`);
      }
      index += 1;
      return value;
    };

    if (arg === '--style') opts.style = nextValue();
    else if (arg === '--citum-style') opts.citumStyle = nextValue();
    else if (arg === '--snapshot') {
      opts.snapshot = path.resolve(nextValue());
      opts.snapshotExplicit = true;
    }
    else if (arg === '--fixture') opts.fixture = path.resolve(nextValue());
    else if (arg === '--output') opts.output = nextValue();
    else if (arg === '--title') opts.title = nextValue();
    else if (arg === '--generate-snapshot') opts.generateSnapshot = true;
    else if (arg === '--force-snapshot') opts.forceSnapshot = true;
    else if (arg === '--bib') opts.bib = path.resolve(nextValue());
    else if (arg === '--biblatex-opts') opts.biblatexOpts = nextValue();
    else if (arg === '--cite') opts.cite = nextValue().split(',').map((value) => value.trim()).filter(Boolean);
    else if (arg === '-h' || arg === '--help') opts.help = true;
    else throw new Error(`Unknown option: ${arg}`);
  }

  if (!opts.citumStyle) opts.citumStyle = opts.style;
  if (!opts.snapshot && opts.citumStyle) {
    opts.snapshot = path.join(DEFAULT_SNAPSHOT_DIR, `${opts.citumStyle}.json`);
  }

  return opts;
}

function repoRelative(filePath) {
  const relative = path.relative(PROJECT_ROOT, path.resolve(filePath));
  return relative && !relative.startsWith('..') ? relative : filePath;
}

function titleCaseStem(stem) {
  return String(stem || '')
    .split(/[-_]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function loadSnapshot(snapshotPath) {
  let snapshot;
  try {
    snapshot = JSON.parse(fs.readFileSync(snapshotPath, 'utf8'));
  } catch (error) {
    throw new Error(`Unable to read snapshot ${repoRelative(snapshotPath)}: ${error.message}`);
  }

  if (!Array.isArray(snapshot.bibliography)) {
    throw new Error(`Snapshot ${repoRelative(snapshotPath)} is missing a bibliography array`);
  }

  return snapshot;
}

function loadFixture(fixturePath) {
  let data;
  try {
    data = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
  } catch (error) {
    throw new Error(`Unable to read fixture ${repoRelative(fixturePath)}: ${error.message}`);
  }

  return Object.values(data).filter((value) => value && typeof value === 'object' && value.id);
}

function defaultSnapshotCommand(opts) {
  const args = ['node', 'scripts/gen-biblatex-snapshot.js', '--style', opts.style];
  if (opts.citumStyle && opts.citumStyle !== opts.style) args.push('--citum-style', opts.citumStyle);
  if (opts.fixture && path.resolve(opts.fixture) !== DEFAULT_FIXTURE) {
    args.push('--fixture', repoRelative(opts.fixture));
  }
  if (opts.bib) args.push('--bib', repoRelative(opts.bib));
  if (opts.biblatexOpts) args.push('--biblatex-opts', opts.biblatexOpts);
  if (opts.cite?.length) args.push('--cite', opts.cite.join(','));
  if (opts.forceSnapshot) args.push('--force');
  return args.join(' ');
}

function ensureSnapshot(opts) {
  if (opts.forceSnapshot || (opts.generateSnapshot && !fs.existsSync(opts.snapshot))) {
    if (opts.snapshotExplicit) {
      throw new Error(
        `Cannot generate a custom --snapshot path: ${repoRelative(opts.snapshot)}\n` +
        'Snapshot generation writes to tests/snapshots/biblatex/<citum-style>.json. ' +
        'Drop --snapshot and use --citum-style to choose that filename, or create the custom snapshot file first.'
      );
    }

    const args = [SNAPSHOT_GENERATOR, '--style', opts.style, '--citum-style', opts.citumStyle];
    if (opts.fixture) args.push('--fixture', opts.fixture);
    if (opts.bib) args.push('--bib', opts.bib);
    if (opts.biblatexOpts) args.push('--biblatex-opts', opts.biblatexOpts);
    if (opts.cite?.length) args.push('--cite', opts.cite.join(','));
    if (opts.forceSnapshot) args.push('--force');

    const result = spawnSync('node', args, {
      cwd: PROJECT_ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    if (result.status !== 0) {
      throw new Error(
        `Snapshot generation failed for ${opts.style}.\n` +
        `${result.stderr || result.stdout || 'No generator output.'}`
      );
    }
    process.stderr.write(result.stderr || result.stdout);
  }

  if (!fs.existsSync(opts.snapshot)) {
    if (opts.snapshotExplicit) {
      throw new Error(
        `Biblatex snapshot not found: ${repoRelative(opts.snapshot)}\n` +
        'Custom --snapshot paths are read-only inputs. Create that file first, ' +
        'or drop --snapshot and use --citum-style with --generate-snapshot to generate ' +
        'tests/snapshots/biblatex/<citum-style>.json.'
      );
    }

    throw new Error(
      `Biblatex snapshot not found: ${repoRelative(opts.snapshot)}\n` +
      `Generate it first:\n  ${defaultSnapshotCommand(opts)}\n` +
      `Or rerun this scaffold command with --generate-snapshot.`
    );
  }
}

function mapEntriesToReferences(snapshot, fixtureRefs, citeOrder = null) {
  if (!Array.isArray(snapshot.bibliography)) {
    throw new Error('Snapshot bibliography must be an array');
  }

  let refs;
  if (citeOrder?.length) {
    const byId = new Map(fixtureRefs.map((ref) => [ref.id, ref]));
    refs = citeOrder.map((id) => {
      const ref = byId.get(id);
      if (!ref) throw new Error(`--cite id not found in fixture: ${id}`);
      return ref;
    });
  } else {
    refs = fixtureRefs.slice(0, snapshot.bibliography.length);
  }

  if (snapshot.bibliography.length !== refs.length) {
    const basis = citeOrder?.length ? '--cite order' : 'fixture order';
    throw new Error(
      `Snapshot/fixture count mismatch using ${basis}: ` +
      `${snapshot.bibliography.length} bibliography entries vs ${refs.length} fixture refs`
    );
  }

  return snapshot.bibliography.map((entry, index) => ({ entry, ref: refs[index] }));
}

function isNumericSnapshot(entries) {
  const labelled = entries.filter(({ entry }) => /^\s*(?:\[\d+\]|\(\d+\)|\d+\.)\s/.test(entry));
  return entries.length > 0 && labelled.length / entries.length >= 0.7;
}

function detectNumericWrap(entries) {
  const counts = { brackets: 0, parentheses: 0 };
  for (const { entry } of entries) {
    if (/^\s*\[\d+\]/.test(entry)) counts.brackets += 1;
    if (/^\s*\(\d+\)/.test(entry)) counts.parentheses += 1;
  }
  return counts.parentheses > counts.brackets ? 'parentheses' : 'brackets';
}

function inferSort(entries) {
  const idsInOutput = entries.map(({ ref }) => ref.id);
  const fixtureOrder = [...idsInOutput].sort((left, right) => {
    const leftNum = Number(String(left).match(/\d+/)?.[0] || 0);
    const rightNum = Number(String(right).match(/\d+/)?.[0] || 0);
    return leftNum - rightNum;
  });

  if (idsInOutput.every((id, index) => id === fixtureOrder[index])) {
    return null;
  }

  return {
    preset: 'bibliography-order',
    comment: 'Rendered order differs from fixture order; confirm the biblatex sorting rule by hand.',
  };
}

function detectNameForm(entries) {
  let initials = 0;
  let full = 0;
  let familyFirst = 0;
  let givenFirst = 0;
  const delimiters = {};
  const ands = { text: 0, symbol: 0 };

  for (const { entry, ref } of entries) {
    const names = ref.author;
    if (!Array.isArray(names) || names.length === 0) continue;
    const first = names.find((name) => name.family && name.given);
    if (!first) continue;

    const normalized = normalizeText(entry);
    const family = first.family;
    const given = first.given;
    const firstGiven = given.split(/\s+/)[0];
    const initialPattern = new RegExp(`(?:^|\\s|,|;)${firstGiven.charAt(0)}\\.?(?:\\s|$).*${escapeRegex(family)}|${escapeRegex(family)}(?:,\\s*)?${firstGiven.charAt(0)}\\.?(?:\\s|$)`, 'i');
    const fullPattern = new RegExp(`${escapeRegex(firstGiven)}\\s+${escapeRegex(family)}|${escapeRegex(family)},\\s*${escapeRegex(firstGiven)}`, 'i');

    if (fullPattern.test(normalized)) full += 1;
    else if (initialPattern.test(normalized)) initials += 1;

    if (new RegExp(`${escapeRegex(family)},\\s*(?:${escapeRegex(firstGiven)}|${firstGiven.charAt(0)}\\.?)`, 'i').test(normalized)) {
      familyFirst += 1;
    } else if (new RegExp(`(?:${escapeRegex(firstGiven)}|${firstGiven.charAt(0)}\\.?)\\s+${escapeRegex(family)}`, 'i').test(normalized)) {
      givenFirst += 1;
    }

    if (names.length >= 2) {
      if (/\s&\s/.test(normalized)) ands.symbol += 1;
      if (/\sand\s/i.test(normalized)) ands.text += 1;
      const delimiter = normalized.includes(';') ? '; ' : ', ';
      delimiters[delimiter] = (delimiters[delimiter] || 0) + 1;
    }
  }

  const options = {};
  if (initials > full) {
    options['name-form'] = 'initials';
    options['initialize-with'] = '. ';
  }
  if (familyFirst > givenFirst) options['display-as-sort'] = 'all';
  if (Object.keys(delimiters).length) {
    options.delimiter = Object.entries(delimiters).sort((a, b) => b[1] - a[1])[0][0];
  }
  if (ands.symbol > ands.text) options.and = 'symbol';
  else if (ands.text > 0) options.and = 'text';

  return {
    component: {
      'name-order': familyFirst > givenFirst ? 'family-first' : 'given-first',
    },
    options,
  };
}

function detectTitleHints(entries) {
  let quoted = 0;
  let sentenceCase = 0;
  let titleCase = 0;

  for (const { entry, ref } of entries) {
    if (!ref.title) continue;
    const normalized = normalizeText(entry);
    const components = parseComponents(entry, ref);
    if (/[“"][^”"]+[”"]/.test(normalized) && components.title.found) quoted += 1;

    const rendered = components.title.found ? components.title.value : ref.title;
    const words = String(rendered).split(/\s+/).filter((word) => /^[A-Za-z]/.test(word));
    if (words.length >= 3) {
      const uppercaseAfterFirst = words.slice(1).filter((word) => /^[A-Z]/.test(word)).length;
      if (uppercaseAfterFirst <= Math.max(1, Math.floor(words.length / 4))) sentenceCase += 1;
      else titleCase += 1;
    }
  }

  return {
    quoted: quoted / Math.max(1, entries.length) >= 0.4,
    titleMode: titleCase > sentenceCase ? 'humanities' : 'scientific',
  };
}

function detectYearPlacement(entries) {
  const placements = { afterAuthor: 0, afterTitle: 0, terminal: 0 };

  for (const { entry, ref } of entries) {
    const order = analyzeOrdering(entry, ref);
    const yearIndex = order.indexOf('year');
    if (yearIndex < 0) continue;
    const contributorIndex = order.indexOf('contributors');
    const titleIndex = order.indexOf('title');
    if (contributorIndex >= 0 && yearIndex === contributorIndex + 1) placements.afterAuthor += 1;
    else if (titleIndex >= 0 && yearIndex > titleIndex) placements.afterTitle += 1;
    if (yearIndex >= order.length - 2) placements.terminal += 1;
  }

  const best = Object.entries(placements).sort((a, b) => b[1] - a[1])[0];
  if (!best || best[1] === 0) return null;
  if (best[0] === 'afterAuthor') return 'after-author';
  if (best[0] === 'terminal') return 'terminal';
  return 'after-title';
}

function consensusOrder(entries) {
  const counts = {};
  for (const { entry, ref } of entries) {
    const order = analyzeOrdering(entry, ref).filter((name) => COMPONENT_MAP[name]);
    for (let index = 0; index < order.length; index += 1) {
      const name = order[index];
      if (!counts[name]) counts[name] = { frequency: 0, positions: [] };
      counts[name].frequency += 1;
      counts[name].positions.push(index);
    }
  }

  return Object.entries(counts)
    .filter(([, data]) => data.frequency / Math.max(1, entries.length) >= 0.25)
    .map(([name, data]) => ({
      name,
      frequency: data.frequency,
      meanPosition: data.positions.reduce((sum, value) => sum + value, 0) / data.positions.length,
    }))
    .sort((left, right) => left.meanPosition - right.meanPosition || right.frequency - left.frequency)
    .map((item) => item.name);
}

function detectSeparator(entries) {
  const counts = {};
  for (const { entry, ref } of entries) {
    for (const detection of detectDelimiters(entry, ref)) {
      const delimiter = detection.delimiter.replace(/[“”"]/g, '');
      if (delimiter.length >= 1 && delimiter.length <= 4 && /^[.,;: ]+$/.test(delimiter)) {
        counts[delimiter] = (counts[delimiter] || 0) + 1;
      }
    }
  }

  const best = Object.entries(counts).sort((a, b) => b[1] - a[1])[0];
  return best ? best[0] : '. ';
}

function detectEntrySuffix(entries) {
  const counts = {};
  for (const { entry } of entries) {
    const match = normalizeText(entry).match(/([.;:,])$/);
    if (match) counts[match[1]] = (counts[match[1]] || 0) + 1;
  }
  const best = Object.entries(counts).sort((a, b) => b[1] - a[1])[0];
  return best && best[1] / Math.max(1, entries.length) >= 0.5 ? best[0] : '.';
}

function buildTemplate(entries, numeric, nameHints, titleHints) {
  const template = [];
  if (numeric) {
    const wrap = detectNumericWrap(entries);
    template.push({
      ...NUMERIC_LABEL_COMPONENT,
      wrap: { punctuation: wrap },
    });
  }

  for (const name of consensusOrder(entries)) {
    const mapped = clone(COMPONENT_MAP[name]);
    if (!mapped) continue;

    if (name === 'contributors') {
      Object.assign(mapped, nameHints.component);
    } else if (name === 'title' && titleHints.quoted) {
      mapped.wrap = { punctuation: 'quotes' };
    }

    if (!template.some((component) => equivalentComponent(component, mapped))) {
      template.push(mapped);
    }
  }

  if (!template.some((component) => component.contributor === 'author')) {
    template.splice(numeric ? 1 : 0, 0, {
      contributor: 'author',
      form: 'long',
      ...nameHints.component,
    });
  }
  if (!template.some((component) => component.title === 'primary')) {
    template.push({ title: 'primary' });
  }

  return template;
}

function buildScaffold({ opts, snapshot, entries }) {
  const numeric = isNumericSnapshot(entries);
  const nameHints = detectNameForm(entries);
  const titleHints = detectTitleHints(entries);
  const sort = inferSort(entries);
  const separator = detectSeparator(entries);
  const entrySuffix = detectEntrySuffix(entries);
  const datePosition = detectYearPlacement(entries);
  const template = buildTemplate(entries, numeric, nameHints, titleHints);

  const style = {
    info: {
      title: opts.title || titleCaseStem(opts.citumStyle),
      id: opts.citumStyle,
      description: `Hand-finish scaffold inferred from biblatex ${opts.style} bibliography output.`,
      'default-locale': 'en-US',
      source: {
        'csl-id': 'https://ctan.org/pkg/biblatex',
        'adapted-by': 'citum-biblatex-scaffold',
        license: 'http://creativecommons.org/licenses/by-sa/3.0/',
        links: [{ href: 'https://ctan.org/pkg/biblatex', rel: 'documentation' }],
      },
    },
    options: {
      processing: numeric ? 'numeric' : 'author-date',
      contributors: nameHints.options,
      titles: titleHints.titleMode,
    },
    citation: numeric
      ? {
          'template-ref': 'numeric-citation',
          wrap: { punctuation: detectNumericWrap(entries) },
          'multi-cite-delimiter': ',',
        }
      : {
          template: [
            { contributor: 'author' },
            { date: 'issued', form: 'year' },
          ],
          wrap: { punctuation: 'parentheses' },
        },
    bibliography: {
      options: {
        contributors: nameHints.options,
        'entry-suffix': entrySuffix,
        separator,
      },
      template,
    },
  };

  if (datePosition) style.bibliography.options['date-position'] = datePosition;
  if (sort) {
    style.bibliography.sort = sort.preset;
  }

  const yamlText = yaml.dump(style, {
    noRefs: true,
    lineWidth: -1,
    sortKeys: false,
  });

  const notes = [
    '# yaml-language-server: $schema=https://citum.github.io/citum-core/schemas/style.json',
    '# WARNING: hand-finish scaffold only; this is not a biblatex converter.',
    '# It was inferred from rendered bibliography text and cannot infer citation macros, type variants, or fallback logic.',
    `# Source snapshot: ${repoRelative(opts.snapshot)}`,
    `# Fixture: ${repoRelative(opts.fixture)}`,
  ];
  if (sort) notes.push(`# Sort note: ${sort.comment}`);
  if (snapshot.generated_by) notes.push(`# Biblatex authority: ${snapshot.generated_by}`);

  return `${notes.join('\n')}\n\n${yamlText}`;
}

function writeOutput(outputPath, content) {
  if (!outputPath || outputPath === '-') {
    process.stdout.write(content);
    return;
  }
  const resolved = path.resolve(outputPath);
  fs.mkdirSync(path.dirname(resolved), { recursive: true });
  fs.writeFileSync(resolved, content, 'utf8');
  process.stderr.write(`Written scaffold: ${repoRelative(resolved)}\n`);
}

function escapeRegex(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function equivalentComponent(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function runCli(argv = process.argv.slice(2)) {
  let opts;
  try {
    opts = parseArgs(argv);
    if (opts.help) {
      printUsage(process.stdout);
      return 0;
    }
    if (!opts.style) {
      printUsage(process.stderr);
      process.stderr.write('\nError: --style is required\n');
      return 2;
    }

    ensureSnapshot(opts);
    const snapshot = loadSnapshot(opts.snapshot);
    const fixtureRefs = loadFixture(opts.fixture);
    const entries = mapEntriesToReferences(snapshot, fixtureRefs, opts.cite);
    const scaffold = buildScaffold({ opts, snapshot, entries });
    writeOutput(opts.output, scaffold);
    return 0;
  } catch (error) {
    process.stderr.write(`Error: ${error.message}\n`);
    return 1;
  }
}

if (require.main === module) {
  process.exit(runCli());
}

module.exports = {
  buildScaffold,
  consensusOrder,
  defaultSnapshotCommand,
  detectEntrySuffix,
  detectNameForm,
  detectNumericWrap,
  detectSeparator,
  detectTitleHints,
  detectYearPlacement,
  ensureSnapshot,
  inferSort,
  isNumericSnapshot,
  loadFixture,
  loadSnapshot,
  mapEntriesToReferences,
  parseArgs,
  runCli,
  titleCaseStem,
};
