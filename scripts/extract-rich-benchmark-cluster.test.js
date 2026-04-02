const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {
  extractRichBenchmarkCluster,
  parseArgs,
  resolveBenchmarkRun,
  selectFixtureItems,
  summarizeIssueBuckets,
} = require('./extract-rich-benchmark-cluster');

test('parseArgs requires style, out-dir, and exactly one selector', () => {
  assert.throws(() => parseArgs([]), /Missing required flag: --style/);
  assert.throws(
    () => parseArgs(['--style', 'chicago-author-date', '--out-dir', '/tmp/x']),
    /One selector is required/
  );
  assert.throws(
    () => parseArgs([
      '--style', 'chicago-author-date',
      '--type', 'entry-dictionary',
      '--ids', 'abc',
      '--out-dir', '/tmp/x',
    ]),
    /mutually exclusive/
  );
});

test('resolveBenchmarkRun selects chicago-author-date scoring bibliography benchmark by default', () => {
  const benchmark = resolveBenchmarkRun('chicago-author-date');

  assert.equal(benchmark.id, 'chicago-zotero-bibliography');
  assert.equal(benchmark.scope, 'bibliography');
  assert.equal(benchmark.runner, 'citeproc-oracle');
  assert.equal(benchmark.countTowardFidelity, true);
  assert.match(benchmark.refsFixture, /tests\/fixtures\/test-items-library\/chicago-18th\.json$/);
});

test('selectFixtureItems filters by type and explicit ids', () => {
  const items = [
    { id: 'a', type: 'entry-dictionary' },
    { id: 'b', type: 'entry-encyclopedia' },
    { id: 'c', type: 'book' },
  ];

  assert.deepEqual(
    selectFixtureItems(items, { types: ['entry-dictionary', 'entry-encyclopedia'] }).map((item) => item.id),
    ['a', 'b']
  );
  assert.deepEqual(
    selectFixtureItems(items, { ids: ['c'] }).map((item) => item.id),
    ['c']
  );
});

test('summarizeIssueBuckets groups bibliography issues by component and issue', () => {
  const summary = summarizeIssueBuckets([
    { match: false, issues: [{ component: 'title', issue: 'missing' }] },
    { match: false, issues: [{ issue: 'missing_entry' }] },
    { match: true, issues: [{ issue: 'ignored' }] },
  ]);

  assert.deepEqual(summary, {
    'title:missing': 1,
    missing_entry: 1,
  });
});

test('extractRichBenchmarkCluster writes reduced fixture and summaries for type selection', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rich-cluster-test-'));

  try {
    const result = extractRichBenchmarkCluster(
      {
        style: 'chicago-author-date',
        types: ['entry-dictionary', 'entry-encyclopedia'],
        ids: null,
        onlyMismatches: true,
        outDir: tempDir,
        benchmark: 'chicago-zotero-bibliography',
      },
      {
        resolveStylePath() {
          return path.join(tempDir, 'mock-style.csl');
        },
        runOracle(stylePath, refsFixture) {
          const fixture = JSON.parse(fs.readFileSync(refsFixture, 'utf8')).items;
          return {
            bibliography: {
              total: fixture.length,
              passed: fixture.length - 1,
              failed: 1,
              entries: fixture.map((item, index) => ({
                index: index + 1,
                oracle: `${item.id}:oracle`,
                citum: index === 0 ? `${item.id}:citum` : `${item.id}:oracle`,
                match: index !== 0,
                issues: index === 0 ? [{ issue: 'missing_entry' }] : [],
              })),
            },
          };
        },
        renderOracleRows(stylePath, items) {
          return items.map((item) => ({
            id: item.id,
            oracleText: `${item.id}:oracle`,
          }));
        },
      }
    );

    const clusterFixture = JSON.parse(fs.readFileSync(result.clusterFixturePath, 'utf8'));
    const clusterBefore = JSON.parse(fs.readFileSync(result.clusterBeforePath, 'utf8'));
    const clusterSummary = JSON.parse(fs.readFileSync(result.clusterSummaryPath, 'utf8'));

    assert.equal(clusterFixture.items.length, 1);
    assert.equal(clusterFixture.items[0].id, '6188419/R362J6UF');
    assert.deepEqual(clusterBefore.selector.types, ['entry-dictionary', 'entry-encyclopedia']);
    assert.equal(clusterBefore.initialSelection.count, 11);
    assert.deepEqual(clusterBefore.mismatchIds, ['6188419/R362J6UF']);
    assert.equal(clusterBefore.unresolvedMismatchCount, 0);
    assert.equal(clusterSummary.initialSummary.total, 11);
    assert.equal(clusterSummary.clusterSummary.total, 1);
    assert.equal(clusterSummary.reductionApplied, true);
    assert.deepEqual(clusterSummary.clusterSummary.issueBuckets, { missing_entry: 1 });
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('extractRichBenchmarkCluster supports explicit id selection without mismatch reduction', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rich-cluster-id-test-'));

  try {
    const result = extractRichBenchmarkCluster(
      {
        style: 'chicago-author-date',
        types: null,
        ids: ['6188419/R362J6UF', '6188419/YGPJER8J'],
        onlyMismatches: false,
        outDir: tempDir,
        benchmark: 'chicago-zotero-bibliography',
      },
      {
        resolveStylePath() {
          return path.join(tempDir, 'mock-style.csl');
        },
        runOracle(stylePath, refsFixture) {
          const fixture = JSON.parse(fs.readFileSync(refsFixture, 'utf8')).items;
          return {
            bibliography: {
              total: fixture.length,
              passed: fixture.length,
              failed: 0,
              entries: fixture.map((item, index) => ({
                index: index + 1,
                oracle: `${item.id}:oracle`,
                citum: `${item.id}:oracle`,
                match: true,
                issues: [],
              })),
            },
          };
        },
        renderOracleRows(stylePath, items) {
          return items.map((item) => ({
            id: item.id,
            oracleText: `${item.id}:oracle`,
          }));
        },
      }
    );

    const clusterFixture = JSON.parse(fs.readFileSync(result.clusterFixturePath, 'utf8'));
    assert.deepEqual(
      clusterFixture.items.map((item) => item.id),
      ['6188419/R362J6UF', '6188419/YGPJER8J']
    );
    const clusterBefore = JSON.parse(fs.readFileSync(result.clusterBeforePath, 'utf8'));
    assert.equal(clusterBefore.selector.types, null);
    assert.deepEqual(clusterBefore.selector.ids, ['6188419/R362J6UF', '6188419/YGPJER8J']);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test('extractRichBenchmarkCluster keeps the selected cluster when mismatches cannot be mapped back to ids', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'rich-cluster-unmapped-test-'));

  try {
    const result = extractRichBenchmarkCluster(
      {
        style: 'chicago-author-date',
        types: ['entry-dictionary', 'entry-encyclopedia'],
        ids: null,
        onlyMismatches: true,
        outDir: tempDir,
        benchmark: 'chicago-zotero-bibliography',
      },
      {
        resolveStylePath() {
          return path.join(tempDir, 'mock-style.csl');
        },
        runOracle(stylePath, refsFixture) {
          const fixture = JSON.parse(fs.readFileSync(refsFixture, 'utf8')).items;
          return {
            bibliography: {
              total: fixture.length,
              passed: fixture.length - 1,
              failed: 1,
              entries: fixture.map((item, index) => (
                index === 0
                  ? {
                    index: 1,
                    oracle: null,
                    citum: `${item.id}:citum`,
                    match: false,
                    issues: [{ issue: 'extra_entry' }],
                  }
                  : {
                    index: index + 1,
                    oracle: `${item.id}:oracle`,
                    citum: `${item.id}:oracle`,
                    match: true,
                    issues: [],
                  }
              )),
            },
          };
        },
        renderOracleRows(stylePath, items) {
          return items.map((item) => ({
            id: item.id,
            oracleText: `${item.id}:oracle`,
          }));
        },
      }
    );

    const clusterFixture = JSON.parse(fs.readFileSync(result.clusterFixturePath, 'utf8'));
    const clusterSummary = JSON.parse(fs.readFileSync(result.clusterSummaryPath, 'utf8'));

    assert.equal(clusterFixture.items.length, 11);
    assert.equal(clusterSummary.reductionApplied, false);
    assert.equal(clusterSummary.unresolvedMismatchCount, 1);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
