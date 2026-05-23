const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const yaml = require('js-yaml');

const {
  buildScaffold,
  detectNameForm,
  isNumericSnapshot,
  mapEntriesToReferences,
  parseArgs,
  runCli,
} = require('./scaffold-biblatex-style');

function makeRefs() {
  return [
    {
      id: 'ITEM-1',
      type: 'article-journal',
      title: 'Deep Learning',
      author: [
        { family: 'LeCun', given: 'Yann' },
        { family: 'Bengio', given: 'Yoshua' },
      ],
      issued: { 'date-parts': [[2015]] },
      'container-title': 'Nature',
      volume: '521',
      page: '436-444',
      DOI: '10.1038/nature14539',
    },
    {
      id: 'ITEM-2',
      type: 'book',
      title: 'A Brief History of Time',
      author: [{ family: 'Hawking', given: 'Stephen' }],
      issued: { 'date-parts': [[1988]] },
      publisher: 'Bantam',
      'publisher-place': 'New York',
    },
  ];
}

function makeSnapshot() {
  return {
    version: 1,
    generated_by: 'biblatex@test+biber@test',
    biblatex_style: 'numeric-comp',
    citum_style: 'numeric-comp',
    bibliography: [
      '[1] Yann LeCun and Yoshua Bengio. “Deep Learning”. In: Nature 521 (2015), pp. 436–444. doi: 10.1038/nature14539.',
      '[2] Stephen Hawking. A Brief History of Time. New York: Bantam, 1988.',
    ],
  };
}

test('parseArgs sets citum style and default snapshot path', () => {
  const opts = parseArgs(['--style', 'alpha', '--output', '/tmp/alpha.yaml']);

  assert.equal(opts.style, 'alpha');
  assert.equal(opts.citumStyle, 'alpha');
  assert.match(opts.snapshot, /tests\/snapshots\/biblatex\/alpha\.json$/);
  assert.equal(opts.output, '/tmp/alpha.yaml');
});

test('mapEntriesToReferences uses cite order when supplied', () => {
  const snapshot = makeSnapshot();
  const refs = makeRefs();
  const mapped = mapEntriesToReferences(snapshot, refs, ['ITEM-2', 'ITEM-1']);

  assert.equal(mapped[0].ref.id, 'ITEM-2');
  assert.equal(mapped[1].ref.id, 'ITEM-1');
});

test('mapEntriesToReferences fails clearly on count mismatch', () => {
  const snapshot = makeSnapshot();
  const refs = makeRefs().slice(0, 1);

  assert.throws(
    () => mapEntriesToReferences(snapshot, refs),
    /Snapshot\/fixture count mismatch using fixture order: 2 bibliography entries vs 1 fixture refs/,
  );
});

test('numeric and contributor hints are inferred from rendered entries', () => {
  const entries = mapEntriesToReferences(makeSnapshot(), makeRefs());
  const nameHints = detectNameForm(entries);

  assert.equal(isNumericSnapshot(entries), true);
  assert.equal(nameHints.component['name-order'], 'given-first');
  assert.equal(nameHints.options.and, 'text');
});

test('buildScaffold emits parseable hand-finish YAML with numeric citation starter', () => {
  const snapshot = makeSnapshot();
  const opts = {
    style: 'numeric-comp',
    citumStyle: 'numeric-comp',
    snapshot: 'tests/snapshots/biblatex/numeric-comp.json',
    fixture: 'tests/fixtures/references-expanded.json',
    title: null,
  };
  const entries = mapEntriesToReferences(snapshot, makeRefs());
  const scaffold = buildScaffold({ opts, snapshot, entries });
  const parsed = yaml.load(scaffold);

  assert.match(scaffold, /WARNING: hand-finish scaffold only/);
  assert.equal(parsed.info.id, 'numeric-comp');
  assert.equal(parsed.options.processing, 'numeric');
  assert.equal(parsed.citation['template-ref'], 'numeric-citation');
  assert.deepEqual(parsed.bibliography.template[0], {
    number: 'citation-number',
    wrap: { punctuation: 'brackets' },
    suffix: ' ',
  });
});

test('runCli reports missing snapshot with generation guidance', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'biblatex-scaffold-test-'));
  const snapshotPath = path.join(tempDir, 'missing.json');
  const fixturePath = path.join(tempDir, 'refs.json');
  fs.writeFileSync(fixturePath, JSON.stringify({ ITEM: makeRefs()[0] }), 'utf8');

  let stderr = '';
  const originalWrite = process.stderr.write;
  process.stderr.write = (chunk) => {
    stderr += String(chunk);
    return true;
  };
  try {
    const status = runCli([
      '--style', 'alpha',
      '--snapshot', snapshotPath,
      '--fixture', fixturePath,
      '--output', '-',
    ]);
    assert.equal(status, 1);
  } finally {
    process.stderr.write = originalWrite;
  }

  assert.match(stderr, /Biblatex snapshot not found/);
  assert.match(stderr, /scripts\/gen-biblatex-snapshot\.js --style alpha/);
  assert.match(stderr, /--generate-snapshot/);
});
