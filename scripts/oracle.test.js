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
  refsDataForProcessor,
} = require('./oracle');
const {
  attachRegisteredDivergenceAdjustments,
  detectDiv004OrderDifference,
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

test('normalizeFixtureItems handles wrapped and array fixtures by item id', () => {
  const wrapped = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'compound-numeric-refs.json'), 'utf8')
  );
  const arrayFixture = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'tests', 'fixtures', 'references-humanities-note.json'), 'utf8')
  );

  const wrappedItems = normalizeFixtureItems(wrapped);
  const arrayItems = normalizeFixtureItems(arrayFixture);

  assert.ok(wrappedItems['zwart1983']);
  assert.ok(wrappedItems['astm-e2881']);
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
      entries: [{ id: 'cite-1', oracle: '[1]', csln: '[2]', match: false }],
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

test('registered divergence adjustments skip order inspection without failures', () => {
  const rawResults = {
    citations: {
      total: 1,
      passed: 1,
      failed: 0,
      entries: [{ id: 'cite-1', oracle: '[1]', csln: '[1]', match: true }],
    },
    bibliography: {
      total: 1,
      passed: 1,
      failed: 0,
      entries: [{ oracle: 'Alpha.', csln: 'Alpha.', match: true }],
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
