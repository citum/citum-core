const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const {
  compareComponents,
  cleanupOracleTempWorkspace,
  createOracleTempWorkspace,
  loadFixtures,
  normalizeFixtureItems,
  parseCitumRenderOutput,
  refsDataForProcessor,
  resolveAuthoredStylePath,
} = require('./oracle');
const {
  attachRegisteredDivergenceAdjustments,
  detectDiv004OrderDifference,
  detectDiv008OrderDifference,
  explainCitationMismatchFromDiv005,
  explainCitationMismatchFromDiv008,
} = require('./lib/oracle-divergences');
const { loadVerificationPolicy, resolveRegisteredDivergence } = require('./lib/verification-policy');

const projectRoot = path.resolve(__dirname, '..');
const oracleScript = path.join(__dirname, 'oracle.js');
const hasLegacyStyles = fs.existsSync(path.join(projectRoot, 'styles-legacy', 'apa.csl'));

function runOracle(stylePath) {
  return new Promise((resolve, reject) => {
    const proc = spawn('node', [oracleScript, stylePath, '--json'], {
      cwd: projectRoot,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    proc.stdout.on('data', (chunk) => {
      stdout += chunk;
    });

    proc.stderr.on('data', (chunk) => {
      stderr += chunk;
    });

    proc.on('error', reject);
    proc.on('close', (code) => {
      if (code === 2) {
        reject(new Error(`oracle fatal for ${stylePath}: ${stderr || stdout}`));
        return;
      }

      try {
        resolve({
          code,
          json: JSON.parse(stdout),
          stderr,
        });
      } catch (error) {
        reject(new Error(`failed to parse oracle output for ${stylePath}: ${error.message}\n${stdout}\n${stderr}`));
      }
    });
  });
}

test('oracle temp workspaces are unique and removable', () => {
  const first = createOracleTempWorkspace();
  const second = createOracleTempWorkspace();

  assert.notEqual(first.dir, second.dir);
  assert.equal(fs.existsSync(first.dir), true);
  assert.equal(fs.existsSync(second.dir), true);

  cleanupOracleTempWorkspace(first);
  cleanupOracleTempWorkspace(second);

  assert.equal(fs.existsSync(first.dir), false);
  assert.equal(fs.existsSync(second.dir), false);
});

test('parallel oracle invocations do not collide on temp files', {
  timeout: 240000,
  skip: !hasLegacyStyles,
}, async () => {
  const styles = [
    path.join(projectRoot, 'styles-legacy', 'american-association-for-cancer-research.csl'),
    path.join(projectRoot, 'styles-legacy', 'american-institute-of-physics.csl'),
  ];

  const results = await Promise.all(styles.map((stylePath) => runOracle(stylePath)));

  for (let i = 0; i < results.length; i += 1) {
    const stylePath = styles[i];
    const expectedStyle = path.basename(stylePath, '.csl');
    const result = results[i];

    assert.ok([0, 1].includes(result.code), `unexpected oracle exit code for ${expectedStyle}: ${result.code}`);
    assert.equal(result.json.style, expectedStyle);
    assert.equal(result.json.error, undefined, `oracle reported fatal error for ${expectedStyle}`);
  }

  for (const tempFile of ['.migrated-refs.json', '.migrated-citations.json', '.migrated-temp.yaml']) {
    assert.equal(
      fs.existsSync(path.join(projectRoot, tempFile)),
      false,
      `legacy shared temp file should not exist: ${tempFile}`
    );
  }
});

test('oracle rejects unsupported citation-only scope', {
  skip: !hasLegacyStyles,
}, async () => {
  const stylePath = path.join(projectRoot, 'styles-legacy', 'apa.csl');

  await new Promise((resolve, reject) => {
    const proc = spawn('node', [oracleScript, stylePath, '--json', '--scope', 'citation'], {
      cwd: projectRoot,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stderr = '';
    proc.stderr.on('data', (chunk) => {
      stderr += chunk;
    });
    proc.on('error', reject);
    proc.on('close', (code) => {
      try {
        assert.equal(code, 1);
        assert.match(stderr, /scope citation is not yet supported/);
        resolve();
      } catch (error) {
        reject(error);
      }
    });
  });
});

test('normalizeFixtureItems handles wrapped and array fixtures by item id', () => {
  const wrapped = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'compound-numeric-refs.json'), 'utf8')
  );
  const zoteroWrapped = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'test-items-library', 'chicago-18th.json'), 'utf8')
  );
  const arrayFixture = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'references-humanities-note.json'), 'utf8')
  );

  const wrappedItems = normalizeFixtureItems(wrapped);
  const zoteroItems = normalizeFixtureItems(zoteroWrapped);
  const arrayItems = normalizeFixtureItems(arrayFixture);

  assert.ok(wrappedItems['zwart1983']);
  assert.ok(wrappedItems['astm-e2881']);
  assert.ok(zoteroItems['6188419/UDAG5V6W']);
  assert.ok(zoteroItems['6188419/SIQ8AUN7']);
  assert.ok(arrayItems['ginzburg1976']);
  assert.ok(arrayItems['foucault-interview']);
});

test('loadFixtures preserves raw wrapped fixtures for processor rendering', () => {
  const { refsData, testItems } = loadFixtures(
    path.join(projectRoot, 'tests', 'fixtures', 'compound-numeric-refs.json'),
    path.join(projectRoot, 'tests', 'fixtures', 'citations-compound-numeric.json')
  );

  assert.ok(Array.isArray(refsData.references));
  assert.ok(refsData.sets);
  assert.ok(testItems['zwart1983']);
  assert.ok(testItems['johnson2021-patent']);
});

test('refsDataForProcessor preserves wrapped fixture sets', () => {
  const wrapped = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'compound-numeric-refs.json'), 'utf8')
  );

  const refsData = refsDataForProcessor(wrapped);

  assert.ok(Array.isArray(refsData.references));
  assert.ok(refsData.sets);
  assert.deepEqual(Object.keys(refsData.sets).sort(), [
    'catalysis-studies',
    'peroxisome-biogenesis',
  ]);
});

test('refsDataForProcessor converts Zotero item wrappers into a processor-ready array', () => {
  const zoteroWrapped = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'test-items-library', 'chicago-18th.json'), 'utf8')
  );

  const refsData = refsDataForProcessor(zoteroWrapped);

  assert.ok(Array.isArray(refsData));
  assert.equal(refsData.length, 403);
  assert.equal(refsData[0].id, '6188419/UDAG5V6W');
  const normalizedDate = refsData.find((item) => item.id === '6188419/TZUZU9ZP')?.issued?.['date-parts']?.[0]?.[0];
  assert.equal(normalizedDate, 2017);
  const normalizedSeason = refsData.find((item) => item.id === '6188419/PTREB4H3')?.issued?.season;
  assert.equal(normalizedSeason, 21);
});

test('div-004 detection recognizes missing-name order drift without treating named order as changed', () => {
  const refsData = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'references-expanded.json'), 'utf8')
  );
  const testItems = Object.fromEntries(
    Object.entries(refsData).filter(([key]) => key !== 'comment')
  );
  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, 'div-004');

  const divergence = detectDiv004OrderDifference(
    [
      'Kuhn, Thomas S. The Structure of Scientific Revolutions. 1962.',
      'Brown v. Board of Education.',
    ],
    ['ITEM-20', 'ITEM-1'],
    {
      'ITEM-1': testItems['ITEM-1'],
      'ITEM-20': testItems['ITEM-20'],
    },
    divergenceRule
  );

  assert.equal(divergence?.divergenceId, 'div-004');
  assert.deepEqual(divergence?.anonymousIds, ['ITEM-20']);
});

test('registered divergence adjustments convert sort-derived numeric label drift into adjusted passes', () => {
  const rawResults = {
    style: 'association-for-computing-machinery',
    citations: {
      total: 1,
      passed: 0,
      failed: 1,
      entries: [{ id: 'cite-1', oracle: '[1]', citum: '[2]', match: false }],
    },
    bibliography: {
      total: 2,
      passed: 2,
      failed: 0,
      entries: [],
    },
    citationsByType: {},
    componentSummary: {},
    orderingIssues: 0,
  };

  const adjusted = attachRegisteredDivergenceAdjustments(
    rawResults,
    [
      'Kuhn, Thomas S. The Structure of Scientific Revolutions. 1962.',
      'Brown v. Board of Education.',
    ],
    ['ITEM-20', 'ITEM-1'],
    {
      'ITEM-1': {
        id: 'ITEM-1',
        type: 'book',
        title: 'The Structure of Scientific Revolutions',
        author: [{ family: 'Kuhn', given: 'Thomas S.' }],
      },
      'ITEM-20': {
        id: 'ITEM-20',
        type: 'legal_case',
        title: 'Brown v. Board of Education',
      },
    },
    [
      {
        id: 'cite-1',
        items: [{ id: 'ITEM-1' }],
      },
    ]
  );

  assert.equal(adjusted.adjusted.citations.passed, 1);
  assert.equal(adjusted.adjusted.citations.failed, 0);
  assert.equal(
    adjusted.adjusted.citations.entries[0].appliedDivergence?.divergenceId,
    'div-004'
  );
});

test('div-005 recognizes structured archival manuscript detail as an intentional citation divergence', () => {
  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, 'div-005');
  const entry = {
    id: 'hn-dead-sea-scrolls',
    oracle: '“The Community Rule (1QS),” Manuscript scroll, 100 BC',
    citum: '“The Community Rule (1QS)”, Manuscript scroll, -100, Shrine of the Book, Israel Antiquities Authority, Jerusalem',
    match: false,
  };
  const citationFixture = {
    id: 'hn-dead-sea-scrolls',
    items: [{ id: 'dead-sea-scrolls' }],
  };
  const testItems = {
    'dead-sea-scrolls': {
      id: 'dead-sea-scrolls',
      type: 'manuscript',
      issued: { 'date-parts': [[-100]] },
      'archive-info': {
        name: 'Israel Antiquities Authority',
        location: 'Shrine of the Book',
        place: 'Jerusalem',
      },
    },
  };

  const adjustment = explainCitationMismatchFromDiv005(
    entry,
    citationFixture,
    testItems,
    divergenceRule
  );

  assert.equal(adjustment?.divergenceId, 'div-005');
  assert.deepEqual(adjustment?.itemIds, ['dead-sea-scrolls']);
});

test('registered divergence adjustments skip order inspection without failures', () => {
  const rawResults = {
    citations: {
      total: 1,
      passed: 1,
      failed: 0,
      entries: [{ id: 'cite-1', oracle: '[1]', citum: '[1]', match: true }],
    },
    bibliography: {
      total: 1,
      passed: 1,
      failed: 0,
      entries: [{ oracle: 'Alpha.', citum: 'Alpha.', match: true }],
    },
  };

  const adjusted = attachRegisteredDivergenceAdjustments(
    rawResults,
    ['Alpha.'],
    ['ITEM-1'],
    {
      'ITEM-1': {
        id: 'ITEM-1',
        type: 'book',
        title: 'Alpha',
        author: [{ family: 'Able' }],
      },
    },
    [{ id: 'cite-1', items: [{ id: 'ITEM-1' }] }]
  );

  assert.equal(adjusted.bibliographyOrder, null);
  assert.equal(adjusted.adjusted.citations.passed, 1);
  assert.deepEqual(adjusted.adjusted.divergenceSummary, {});
});

test('div-008 detection identifies same-family named items reversed in citum output', () => {
  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, 'div-008');

  const testItems = {
    'ITEM-A': { id: 'ITEM-A', type: 'article-journal', title: 'Given-First Paper', author: [{ family: 'Smith', given: 'Jane' }] },
    'ITEM-B': { id: 'ITEM-B', type: 'article-journal', title: 'Title-First Paper', author: [{ family: 'Smith', given: 'Patricia' }] },
  };

  // Oracle: Jane before Patricia (given-name J < P)
  // Citum:  Patricia before Jane (title G < T)
  const divergence = detectDiv008OrderDifference(
    ['Smith, Jane. Given-First Paper.', 'Smith, Patricia. Title-First Paper.'],
    ['ITEM-B', 'ITEM-A'],
    testItems,
    divergenceRule
  );

  assert.equal(divergence?.divergenceId, 'div-008');
  assert.deepEqual(divergence?.swappedPairs, [['ITEM-A', 'ITEM-B']]);
  assert.deepEqual(divergence?.affectedIds, ['ITEM-A', 'ITEM-B']);
});

test('div-008 detection fires when same-family items are separated by an anonymous item', () => {
  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, 'div-008');

  const testItems = {
    'ITEM-A': { id: 'ITEM-A', type: 'article-journal', title: 'Given-First Paper', author: [{ family: 'Smith', given: 'Jane' }] },
    'ITEM-ANON': { id: 'ITEM-ANON', type: 'legal_case', title: 'Anonymous Case' },
    'ITEM-B': { id: 'ITEM-B', type: 'article-journal', title: 'Title-First Paper', author: [{ family: 'Smith', given: 'Patricia' }] },
  };

  // Oracle named-only order: Jane, Patricia — but an anonymous item sits between them in full order.
  const divergence = detectDiv008OrderDifference(
    [
      'Smith, Jane. Given-First Paper.',
      'Anonymous Case.',
      'Smith, Patricia. Title-First Paper.',
    ],
    ['ITEM-B', 'ITEM-ANON', 'ITEM-A'],
    testItems,
    divergenceRule
  );

  assert.equal(divergence?.divergenceId, 'div-008');
  assert.deepEqual(divergence?.swappedPairs, [['ITEM-A', 'ITEM-B']]);
});

test('div-008 detection returns null when oracle and citum order agree', () => {
  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, 'div-008');

  const testItems = {
    'ITEM-A': { id: 'ITEM-A', type: 'article-journal', title: 'Alpha', author: [{ family: 'Smith', given: 'Jane' }] },
    'ITEM-B': { id: 'ITEM-B', type: 'article-journal', title: 'Beta', author: [{ family: 'Smith', given: 'Patricia' }] },
  };

  const result = detectDiv008OrderDifference(
    ['Smith, Jane. Alpha.', 'Smith, Patricia. Beta.'],
    ['ITEM-A', 'ITEM-B'],
    testItems,
    divergenceRule
  );

  assert.equal(result, null);
});

test('explainCitationMismatchFromDiv008 masks labels for swapped same-family items', () => {
  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, 'div-008');

  const testItems = {
    'ITEM-A': { id: 'ITEM-A', type: 'article-journal', title: 'Given-First', author: [{ family: 'Smith', given: 'Jane' }] },
    'ITEM-B': { id: 'ITEM-B', type: 'article-journal', title: 'Title-First', author: [{ family: 'Smith', given: 'Patricia' }] },
  };

  const div008Info = detectDiv008OrderDifference(
    ['Smith, Jane. Given-First.', 'Smith, Patricia. Title-First.'],
    ['ITEM-B', 'ITEM-A'],
    testItems,
    divergenceRule
  );

  // oracle: ITEM-A=[1], ITEM-B=[2]; citum: ITEM-B=[1], ITEM-A=[2]
  const entry = { id: 'cite-smith', oracle: '[1]', citum: '[2]', match: false };
  const citationFixture = { id: 'cite-smith', items: [{ id: 'ITEM-A' }] };

  const adjustment = explainCitationMismatchFromDiv008(entry, citationFixture, div008Info);

  assert.equal(adjustment?.divergenceId, 'div-008');
  assert.deepEqual(adjustment?.itemIds, ['ITEM-A']);
  assert.deepEqual(adjustment?.oracleLabels, [1]);
  assert.deepEqual(adjustment?.citumLabels, [2]);
});

test('div-008 and div-004 fire independently when both conditions are present', () => {
  const rawResults = {
    style: 'acm-sig-proceedings',
    citations: {
      total: 2,
      passed: 0,
      failed: 2,
      entries: [
        { id: 'cite-smith-jane', oracle: '[1]', citum: '[2]', match: false },
        { id: 'cite-anon', oracle: '[3]', citum: '[2]', match: false },
      ],
    },
    bibliography: {
      total: 3,
      passed: 3,
      failed: 0,
      entries: [],
    },
    citationsByType: {},
    componentSummary: {},
    orderingIssues: 0,
  };

  const testItems = {
    'ITEM-A': { id: 'ITEM-A', type: 'article-journal', title: 'Given-First', author: [{ family: 'Smith', given: 'Jane' }] },
    'ITEM-B': { id: 'ITEM-B', type: 'article-journal', title: 'Title-First', author: [{ family: 'Smith', given: 'Patricia' }] },
    'ITEM-ANON': { id: 'ITEM-ANON', type: 'legal_case', title: 'Anon Case' },
  };

  // Oracle: Jane[1], Patricia[2], Anon[3]
  // Citum:  Patricia[1], Jane[2], Anon[3]  ← div-008 (Smith swap); anon stays last
  const adjusted = attachRegisteredDivergenceAdjustments(
    rawResults,
    [
      'Smith, Jane. Given-First.',
      'Smith, Patricia. Title-First.',
      'Anon Case.',
    ],
    ['ITEM-B', 'ITEM-A', 'ITEM-ANON'],
    testItems,
    [
      { id: 'cite-smith-jane', items: [{ id: 'ITEM-A' }] },
      { id: 'cite-anon', items: [{ id: 'ITEM-ANON' }] },
    ]
  );

  const summary = adjusted.adjusted.divergenceSummary;
  // Both divergences must appear in the summary independently.
  assert.ok(summary['div-008'], 'div-008 should be in divergence summary');
  assert.ok(summary['div-004'], 'div-004 should also be in divergence summary');
  assert.deepEqual(summary['div-008'].swappedPairs, [['ITEM-A', 'ITEM-B']]);

  // The Smith Jane citation is explained by whichever divergence fires first —
  // both produce equivalent label masking; the exact divergenceId is not load-bearing.
  const smithEntry = adjusted.adjusted.citations.entries[0];
  assert.equal(smithEntry.match, true, 'a registered divergence should explain the Smith citation mismatch');
  assert.ok(
    ['div-004', 'div-008'].includes(smithEntry.appliedDivergence?.divergenceId),
    `applied divergence should be div-004 or div-008, got: ${smithEntry.appliedDivergence?.divergenceId}`
  );
});

test('resolveAuthoredStylePath prefers styles/<name>.yaml over embedded', () => {
  const dir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'oracle-resolve-'));
  try {
    fs.mkdirSync(path.join(dir, 'embedded'));
    fs.writeFileSync(path.join(dir, 'acme.yaml'), 'info: {}');
    fs.writeFileSync(path.join(dir, 'embedded', 'acme.yaml'), 'info: {}');

    const resolved = resolveAuthoredStylePath(dir, 'acme');
    assert.equal(resolved, path.join(dir, 'acme.yaml'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('resolveAuthoredStylePath falls back to styles/embedded/<name>.yaml', () => {
  const dir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'oracle-resolve-'));
  try {
    fs.mkdirSync(path.join(dir, 'embedded'));
    fs.writeFileSync(path.join(dir, 'embedded', 'beta.yaml'), 'info: {}');

    const resolved = resolveAuthoredStylePath(dir, 'beta');
    assert.equal(resolved, path.join(dir, 'embedded', 'beta.yaml'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('resolveAuthoredStylePath returns null when no authored YAML exists', () => {
  const dir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'oracle-resolve-'));
  try {
    fs.mkdirSync(path.join(dir, 'embedded'));
    const resolved = resolveAuthoredStylePath(dir, 'missing');
    assert.equal(resolved, null);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('resolveAuthoredStylePath finds versioned name in embedded/ via prefix scan', () => {
  // 'apa' should find 'apa-7th.yaml' in embedded/ when no exact match exists
  const dir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'oracle-resolve-'));
  try {
    const embeddedDir = path.join(dir, 'embedded');
    fs.mkdirSync(embeddedDir);
    fs.writeFileSync(path.join(embeddedDir, 'apa-7th.yaml'), '{}');
    const resolved = resolveAuthoredStylePath(dir, 'apa');
    assert.equal(resolved, path.join(embeddedDir, 'apa-7th.yaml'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('resolveAuthoredStylePath prefers embedded/ prefix over styles/ prefix', () => {
  // A root-level 'apa-variant.yaml' must not shadow embedded 'apa-7th.yaml'.
  const dir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'oracle-resolve-'));
  try {
    const embeddedDir = path.join(dir, 'embedded');
    fs.mkdirSync(embeddedDir);
    fs.writeFileSync(path.join(embeddedDir, 'apa-7th.yaml'), '{}');
    fs.writeFileSync(path.join(dir, 'apa-classic.yaml'), '{}');
    // embedded prefix wins over root prefix
    const resolved = resolveAuthoredStylePath(dir, 'apa');
    assert.equal(resolved, path.join(embeddedDir, 'apa-7th.yaml'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('resolveAuthoredStylePath picks the first embedded prefix match deterministically', () => {
  const dir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'oracle-resolve-'));
  try {
    const embeddedDir = path.join(dir, 'embedded');
    fs.mkdirSync(embeddedDir);
    fs.writeFileSync(path.join(embeddedDir, 'apa-7th.yaml'), '{}');
    fs.writeFileSync(path.join(embeddedDir, 'apa-6th.yaml'), '{}');
    const resolved = resolveAuthoredStylePath(dir, 'apa');
    assert.equal(resolved, path.join(embeddedDir, 'apa-6th.yaml'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('parseCitumRenderOutput keeps integral citations separate from keyed citations', () => {
  const parsed = parseCitumRenderOutput(
    [
      '=== apa-7th.yaml ===',
      '',
      'CITATIONS (Non-Integral):',
      '  [single-item] (Kuhn, 1962)',
      '',
      'CITATIONS (Integral):',
      '  Kuhn (1962)',
      '',
      'BIBLIOGRAPHY:',
      '  [ITEM-1] Kuhn, T. S. (1962). _The Structure of Scientific Revolutions_.',
      '',
    ].join('\n'),
    { 'ITEM-1': { id: 'ITEM-1' } }
  );

  assert.deepEqual(parsed.citations, { 'single-item': '(Kuhn, 1962)' });
  assert.deepEqual(parsed.integralCitations, ['Kuhn (1962)']);
  assert.deepEqual(parsed.bibliographyOrderIds, ['ITEM-1']);
  assert.deepEqual(parsed.bibliography, [
    'Kuhn, T. S. (1962). _The Structure of Scientific Revolutions_.',
  ]);
});

test('compareComponents reports differing component values as mismatches', () => {
  const { differences, matches } = compareComponents(
    {
      title: { found: true, value: 'Alpha' },
      contributors: { found: false },
      year: { found: false },
      containerTitle: { found: false },
      volume: { found: false },
      issue: { found: false },
      pages: { found: false },
      publisher: { found: false },
      doi: { found: false },
      edition: { found: false },
      editors: { found: false },
      translators: { found: false },
      interviewers: { found: false },
      recipients: { found: false },
    },
    {
      title: { found: true, value: 'Beta' },
      contributors: { found: false },
      year: { found: false },
      containerTitle: { found: false },
      volume: { found: false },
      issue: { found: false },
      pages: { found: false },
      publisher: { found: false },
      doi: { found: false },
      edition: { found: false },
      editors: { found: false },
      translators: { found: false },
      interviewers: { found: false },
      recipients: { found: false },
    },
    {}
  );

  assert.deepEqual(matches, []);
  assert.deepEqual(differences, [{
    component: 'title',
    issue: 'value_mismatch',
    expected: 'Alpha',
    found: 'Beta',
    detail: 'Value differs between oracle and Citum',
  }]);
});
