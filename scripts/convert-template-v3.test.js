const test = require('node:test');
const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {
  convertStyle,
  deriveTemplateVariantDiff,
} = require('./convert-template-v3');

test('deriveTemplateVariantDiff emits rendering modify plus structural operations', () => {
  const base = [
    { contributor: 'author' },
    { title: 'primary' },
    { variable: 'publisher' },
  ];
  const target = [
    { contributor: 'author' },
    { date: 'issued', form: 'year' },
    { title: 'primary', suffix: '.' },
  ];

  const diff = deriveTemplateVariantDiff(base, target);

  assert.deepEqual(diff, {
    modify: [
      { match: { title: 'primary' }, suffix: '.' },
    ],
    remove: [
      { match: { variable: 'publisher' } },
    ],
    add: [
      { component: { date: 'issued', form: 'year' }, before: { title: 'primary' } },
    ],
  });
});

test('deriveTemplateVariantDiff returns null for non-rendering mutations', () => {
  const diff = deriveTemplateVariantDiff(
    [{ title: 'primary' }],
    [{ title: 'primary', form: 'short' }]
  );

  assert.equal(diff, null);
});

test('deriveTemplateVariantDiff can select grouped components', () => {
  const base = [
    {
      delimiter: '',
      group: [
        { number: 'citation-number', wrap: { punctuation: 'brackets' } },
        { contributor: 'author', form: 'long' },
      ],
    },
    { title: 'primary' },
  ];
  const target = [
    {
      delimiter: '',
      group: [
        { number: 'citation-number', wrap: { punctuation: 'brackets' } },
        { contributor: 'author', form: 'long' },
      ],
      suffix: '.',
    },
    { title: 'primary' },
  ];

  const diff = deriveTemplateVariantDiff(base, target);

  assert.deepEqual(diff, {
    modify: [
      { match: { group: base[0].group }, suffix: '.' },
    ],
  });
});

test('convertStyle rewrites only exact diff-compatible variants', () => {
  const base = [
    { contributor: 'author' },
    { title: 'primary' },
    { variable: 'publisher' },
    { variable: 'publisher-place' },
    { date: 'issued' },
    { number: 'pages' },
  ];
  const style = {
    bibliography: {
      template: base,
      'type-variants': {
        book: [
          { contributor: 'author' },
          { title: 'primary', suffix: '.' },
          { variable: 'publisher' },
          { variable: 'publisher-place' },
          { date: 'issued' },
          { number: 'pages' },
        ],
        webpage: [
          { contributor: 'author' },
          { title: 'primary', form: 'short' },
          { variable: 'publisher' },
          { variable: 'publisher-place' },
          { date: 'issued' },
          { number: 'pages' },
        ],
      },
    },
  };

  const summary = convertStyle(style);

  assert.equal(summary.converted, 1);
  assert.equal(summary.total, 2);
  assert.equal(summary.log.length, 1);
  assert.deepEqual(style.bibliography['type-variants'].book, {
    modify: [
      { match: { title: 'primary' }, suffix: '.' },
    ],
  });
  assert.deepEqual(style.bibliography['type-variants'].webpage, [
    { contributor: 'author' },
    { title: 'primary', form: 'short' },
    { variable: 'publisher' },
    { variable: 'publisher-place' },
    { date: 'issued' },
    { number: 'pages' },
  ]);
});

test('convertStyle derives variants from earlier same-section parents', () => {
  const style = {
    bibliography: {
      template: [
        { contributor: 'author' },
        { title: 'primary' },
        { variable: 'publisher' },
        { variable: 'publisher-place' },
        { date: 'issued' },
      ],
      'type-variants': {
        book: [
          { contributor: 'author' },
          { title: 'primary', suffix: '.' },
          { variable: 'publisher' },
          { variable: 'publisher-place' },
          { date: 'issued' },
        ],
        chapter: [
          { contributor: 'author' },
          { title: 'primary', suffix: '!' },
          { variable: 'publisher' },
          { variable: 'publisher-place' },
          { date: 'issued' },
        ],
      },
    },
  };

  const summary = convertStyle(style);

  assert.equal(summary.converted, 2);
  assert.equal(summary.total, 2);
  assert.equal(summary.log.length, 2);
  assert.deepEqual(style.bibliography['type-variants'].book, {
    modify: [
      { match: { title: 'primary' }, suffix: '.' },
    ],
  });
  assert.deepEqual(style.bibliography['type-variants'].chapter, {
    modify: [
      { match: { title: 'primary' }, suffix: '!' },
    ],
  });
});

test('convertStyle skips extending styles to preserve inherited variant overlays', () => {
  const style = {
    extends: 'base-style',
    bibliography: {
      template: [
        { contributor: 'author' },
        { title: 'primary' },
      ],
      'type-variants': {
        book: [
          { contributor: 'author' },
          { title: 'primary', suffix: '.' },
        ],
      },
    },
  };

  const summary = convertStyle(style);

  assert.equal(summary.converted, 0);
  assert.equal(summary.total, 0);
  assert.equal(summary.log.length, 0);
  assert.deepEqual(style.bibliography['type-variants'].book, [
    { contributor: 'author' },
    { title: 'primary', suffix: '.' },
  ]);
});

test('cli summary includes scanned styles with no safe conversions', () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'template-v3-'));
  const stylePath = path.join(tmpDir, 'fallback.yaml');
  fs.writeFileSync(stylePath, `
bibliography:
  template:
    - title: primary
  type-variants:
    book:
      - title: primary
        form: short
`);

  const output = execFileSync(
    process.execPath,
    [path.join(__dirname, 'convert-template-v3.js'), '--json', stylePath],
    { encoding: 'utf8' }
  );
  const summary = JSON.parse(output);

  assert.deepEqual(summary.totals, { files: 1, converted: 0, total: 1 });
  assert.equal(summary.summaries[0].converted, 0);
});

test('--write modifies only type-variants block, leaving other lines unchanged', () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'citum-test-'));
  const filePath = path.join(tmpDir, 'test.yaml');

  const input = [
    'info:',
    '  title: Test Style',
    '  categories:',
    '    - science',
    'options:',
    '  initialize-with: ". "',
    'bibliography:',
    '  template:',
    '    - contributor: author',
    '    - title: primary',
    '    - variable: publisher',
    '    - variable: publisher-place',
    '    - date: issued',
    '    - number: pages',
    '  type-variants:',
    '    book:',
    '      - contributor: author',
    '      - title: primary',
    '        suffix: .',
    '      - variable: publisher',
    '      - variable: publisher-place',
    '      - date: issued',
    '      - number: pages',
    '    webpage:',
    '      - contributor: author',
    '      - title: primary',
    '        form: short',
    '      - variable: publisher',
    '      - variable: publisher-place',
    '      - date: issued',
    '      - number: pages',
  ].join('\n');

  fs.writeFileSync(filePath, input, 'utf8');

  execFileSync(process.execPath, [
    path.join(__dirname, 'convert-template-v3.js'),
    '--write',
    filePath,
  ]);

  const output = fs.readFileSync(filePath, 'utf8');
  const inputLines = input.split('\n');
  const outputLines = output.split('\n');

  // The info/options/bibliography template sections must be byte-for-byte identical
  // Lines 0-14 (before type-variants) must be unchanged
  for (let i = 0; i <= 14; i += 1) {
    assert.equal(outputLines[i], inputLines[i], `Line ${i} changed unexpectedly`);
  }

  // The converted variant (book) must be in diff form
  const bookStart = outputLines.findIndex((l) => /^\s{4}book:/.test(l));
  assert.ok(bookStart >= 0, 'book: key must exist');
  const bookBlock = outputLines.slice(bookStart, bookStart + 5).join('\n');
  assert.ok(bookBlock.includes('modify:'), 'book must have modify: diff');

  // The non-convertible variant (webpage) must be unchanged (still array form)
  const webpageStart = outputLines.findIndex((l) => /^\s{4}webpage:/.test(l));
  assert.ok(webpageStart >= 0, 'webpage: key must exist');
  const webpageLine = outputLines[webpageStart + 1];
  assert.ok(webpageLine.trim().startsWith('-'), 'webpage must remain as array');
});
