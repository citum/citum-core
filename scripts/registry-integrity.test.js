const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const yaml = require('js-yaml');

const WORKSPACE_ROOT = path.resolve(__dirname, '..');
const REGISTRY_PATH = path.join(
  WORKSPACE_ROOT,
  'crates',
  'citum-schema-style',
  'embedded',
  'registry',
  'default.yaml'
);
const CORE_URL_PATTERN =
  /^https:\/\/raw\.githubusercontent\.com\/citum\/citum-core\/[^/]+\/styles\/(.+)$/;
const STYLES_ROOT = path.join(WORKSPACE_ROOT, 'styles');

// Regression guard for the 2026-07 community-corpus split (see
// docs/architecture/audits/): 123 registry entries kept pointing at
// citum-core paths for styles that had moved to citum/citum-styles, so
// every non-embedded registry fetch for those ids 404'd. This asserts every
// citum-core-hosted registry URL still resolves to a real file on disk, so
// that class of breakage cannot recur silently.
test('every citum-core registry URL resolves to a file that exists under styles/', () => {
  const doc = yaml.load(fs.readFileSync(REGISTRY_PATH, 'utf8'));
  assert.ok(Array.isArray(doc.styles), 'registry default.yaml must have a styles array');

  const missing = [];
  for (const entry of doc.styles) {
    const match = entry.url?.match(CORE_URL_PATTERN);
    if (!match) continue;
    const relativePath = match[1];
    const absolutePath = path.join(STYLES_ROOT, relativePath);
    if (!fs.existsSync(absolutePath)) {
      missing.push(`${entry.id} -> ${relativePath}`);
    }
  }

  assert.deepEqual(
    missing,
    [],
    `registry entries point at missing citum-core style files:\n${missing.join('\n')}`
  );
});

test('remote registry URLs use immutable revisions', () => {
  const doc = yaml.load(fs.readFileSync(REGISTRY_PATH, 'utf8'));
  const mutable = doc.styles
    .filter((entry) => entry.url && /\/(?:main|master)\//.test(entry.url))
    .map((entry) => `${entry.id} -> ${entry.url}`);

  assert.deepEqual(mutable, []);
});

test('MHRA is an offline builtin and its public copy stays synchronized', () => {
  const doc = yaml.load(fs.readFileSync(REGISTRY_PATH, 'utf8'));
  const mhra = doc.styles.find((entry) => entry.id === 'mhra-notes');
  assert.deepEqual(mhra, {
    id: 'mhra-notes',
    aliases: ['mhra'],
    builtin: 'mhra-notes',
    kind: 'independent',
    title: 'MHRA Style Guide 4th edition (notes)',
  });

  const publicStyle = fs.readFileSync(path.join(STYLES_ROOT, 'mhra-notes.yaml'));
  const embeddedStyle = fs.readFileSync(
    path.join(
      WORKSPACE_ROOT,
      'crates',
      'citum-schema-style',
      'embedded',
      'styles',
      'mhra-notes.yaml'
    )
  );
  assert.deepEqual(embeddedStyle, publicStyle);
});
