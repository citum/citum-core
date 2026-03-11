const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {
  resolveRegisteredDivergence,
  loadVerificationPolicy,
  resolveVerificationPolicy,
  resolveScopeAuthority,
} = require('./lib/verification-policy');
const { getEffectiveVerificationScopes } = require('./lib/style-verification');
const { loadReportProvenance } = require('./lib/report-metadata');
const {
  buildNoteStyleLookup,
  discoverCoreStyles,
  computeFidelityScore,
  equivalentText,
  expandCompoundBibEntries,
  formatAuthorityLabel,
  getEffectiveOracleSection,
  getCslSnapshotStatus,
  getComparisonEntryTexts,
  mapWithConcurrency,
  mergeDivergenceSummaries,
  preflightSnapshots,
  runCachedJsonJob,
  selectPrimaryComparator,
} = require('./report-core');

const projectRoot = path.resolve(__dirname, '..');
const hasLegacyStyles = fs.existsSync(path.join(projectRoot, 'styles-legacy', 'apa.csl'));

function loadStyleMap() {
  return new Map(discoverCoreStyles().map((style) => [style.name, style]));
}

test('discoverCoreStyles classifies representative style origins and CSL reach', () => {
  const styles = loadStyleMap();
  const provenance = loadReportProvenance();

  assert.equal(styles.get('apa-7th').originLabel, provenance.defaults.labels['csl-derived']);
  assert.equal(styles.get('apa-7th').cslReach, 783);

  assert.equal(styles.get('chem-acs').originLabel, provenance.defaults.labels['biblatex-derived']);
  assert.equal(styles.get('chem-acs').cslReach, null);

  assert.equal(styles.get('numeric-comp').originLabel, provenance.defaults.labels['biblatex-derived']);
  assert.equal(styles.get('numeric-comp').cslReach, null);

  const unknownOrigins = [...styles.values()].filter((style) => style.originLabel === 'Unknown');
  assert.deepEqual(unknownOrigins, []);
});

test('buildNoteStyleLookup indexes shipped note styles', () => {
  const noteStyles = buildNoteStyleLookup();

  assert.equal(noteStyles.has('chicago-notes'), true);
  assert.equal(noteStyles.get('chicago-notes').style.options.processing, 'note');
  assert.equal(noteStyles.has('apa-7th'), false);
});

test('report-core exposes expected benchmark labels for representative styles', () => {
  const styles = loadStyleMap();
  const policy = loadVerificationPolicy();

  const cases = [
    ['apa-7th', 'citeproc-js', null],
    ['chem-acs', 'biblatex: chem-acs', 'chem-acs'],
    ['numeric-comp', 'biblatex: numeric-comp', 'numeric-comp'],
  ];

  for (const [styleName, expectedLabel, authorityId] of cases) {
    const style = styles.get(styleName);
    const stylePolicy = resolveVerificationPolicy(styleName, policy);
    const comparator = selectPrimaryComparator(style, stylePolicy);
    assert.equal(formatAuthorityLabel(comparator, authorityId || stylePolicy.authorityId), expectedLabel);
  }
});

test('verification policy exposes scope-specific authority for chemistry benchmark rows', () => {
  const policy = loadVerificationPolicy();
  const stylePolicy = resolveVerificationPolicy('chem-acs', policy);

  assert.equal(stylePolicy.authority, 'biblatex');
  assert.equal(stylePolicy.authorityId, 'chem-acs');
  assert.equal(stylePolicy.regressionBaseline, 'citum-baseline');

  const citationAuthority = resolveScopeAuthority(stylePolicy, 'citation');
  const bibliographyAuthority = resolveScopeAuthority(stylePolicy, 'bibliography');

  assert.deepEqual(citationAuthority, {
    authority: 'citeproc-js',
    authorityId: null,
    note: stylePolicy.note,
  });
  assert.deepEqual(bibliographyAuthority, {
    authority: 'biblatex',
    authorityId: 'chem-acs',
    note: stylePolicy.note,
  });
});

test('verification policy exposes registered divergence metadata for div-004', () => {
  const policy = loadVerificationPolicy();
  const divergence = resolveRegisteredDivergence(policy, 'div-004');

  assert.deepEqual(divergence.scopes, ['citation', 'bibliography']);
  assert.deepEqual(divergence.tags, [
    'missing-name-title-sort-order',
    'sort-derived-numeric-citation-label',
  ]);
  assert.match(divergence.note, /Missing-name works sort by title/);
});

test('citation-only styles do not advertise bibliography verification scopes', () => {
  const styles = loadStyleMap();
  const policy = loadVerificationPolicy();
  const chicagoNotes = styles.get('chicago-notes');
  const stylePolicy = resolveVerificationPolicy('chicago-notes', policy);

  assert.equal(chicagoNotes.hasBibliography, false);
  assert.deepEqual(getEffectiveVerificationScopes(stylePolicy, chicagoNotes.hasBibliography), ['citation']);
});

test('comparison text helper supports both live-oracle and native-snapshot entry shapes', () => {
  assert.deepEqual(
    getComparisonEntryTexts({ oracle: 'benchmark text', csln: 'citum text' }),
    { benchmark: 'benchmark text', citum: 'citum text' }
  );
  assert.deepEqual(
    getComparisonEntryTexts({ expected: 'snapshot benchmark', actual: 'snapshot citum' }),
    { benchmark: 'snapshot benchmark', citum: 'snapshot citum' }
  );
});

test('effective oracle sections and fidelity prefer adjusted counts when present', () => {
  const oracleResult = {
    citations: { passed: 8, total: 10, entries: [] },
    bibliography: { passed: 9, total: 10, entries: [] },
    adjusted: {
      citations: { passed: 10, total: 10, entries: [] },
      bibliography: { passed: 10, total: 10, entries: [] },
      divergenceSummary: {
        'div-004': { adjustedCitations: 2 },
      },
    },
  };

  assert.deepEqual(getEffectiveOracleSection(oracleResult, 'citations'), oracleResult.adjusted.citations);
  assert.equal(computeFidelityScore(oracleResult), 1);
});

test('mergeDivergenceSummaries preserves counts and unions arrays', () => {
  const merged = mergeDivergenceSummaries(
    {
      'div-004': {
        adjustedCitations: 1,
        bibliographyOrderDifference: true,
        anonymousIds: ['ITEM-20'],
        tags: ['missing-name-title-sort'],
      },
    },
    {
      'div-004': {
        adjustedCitations: 2,
        bibliographyOrderDifference: false,
        anonymousIds: ['ITEM-21'],
        tags: ['sort-derived-numeric-citation-label'],
      },
    }
  );

  assert.deepEqual(merged, {
    'div-004': {
      adjustedCitations: 3,
      bibliographyOrderDifference: true,
      anonymousIds: ['ITEM-20', 'ITEM-21'],
      tags: ['missing-name-title-sort', 'sort-derived-numeric-citation-label'],
    },
  });
});

test('expandCompoundBibEntries splits merged biblatex compound blocks', () => {
  const entries = [
    '(1) First entry. (2) Second entry. (3) Third entry.',
    '(4) Standalone entry.',
  ];

  assert.deepEqual(expandCompoundBibEntries(entries), [
    '(1) First entry.',
    '(2) Second entry.',
    '(3) Third entry.',
    '(4) Standalone entry.',
  ]);
});

test('equivalentText tolerates near-match snapshot formatting without masking drift', () => {
  assert.equal(
    equivalentText(
      '[3] Yann LeCun, Yoshua Bengio, and Geoffrey Hinton. “Deep Learning”. In: Nature 521 (2015), pp. 436–444.',
      '[3] Y. LeCun, Y. Bengio and G. Hinton, “Deep Learning”, Nature, 2015, 521, 436–444.'
    ),
    true
  );

  assert.equal(
    equivalentText(
      '[29] John Smith et al. “Adaptive Climate Risk Modeling in Coastal Cities”. In: Journal of Climate Analytics 12.2 (2021), pp. 101–119.',
      '[30] John Smith et al. “Adaptive Climate Risk Modeling for Inland Regions”. In: Journal of Climate Analytics 12.3 (2021), pp. 201–219.'
    ),
    false
  );
});

test('getCslSnapshotStatus reports missing and stale snapshots without invoking live oracle', () => {
  const refsFixture = path.join(projectRoot, 'tests', 'fixtures', 'references-expanded.json');
  const citationsFixture = path.join(projectRoot, 'tests', 'fixtures', 'citations-expanded.json');
  const missing = getCslSnapshotStatus('/tmp/definitely-missing-style.csl', refsFixture, citationsFixture);
  assert.equal(missing.ok, false);
  assert.equal(missing.status, 'missing');

  const staleCitationsFixture = path.join(os.tmpdir(), `citations-stale-${process.pid}.json`);
  fs.writeFileSync(staleCitationsFixture, fs.readFileSync(citationsFixture, 'utf8').replace('"id":', '"fixture-id":'));
  if (hasLegacyStyles) {
    const stale = getCslSnapshotStatus(
      path.join(projectRoot, 'styles-legacy', 'apa.csl'),
      refsFixture,
      staleCitationsFixture
    );
    assert.equal(stale.ok, false);
    assert.equal(stale.status, 'stale');
  }
  fs.rmSync(staleCitationsFixture, { force: true });
});

test('runCachedJsonJob invalidates when cache key changes', async () => {
  const cacheDir = fs.mkdtempSync(path.join(os.tmpdir(), 'report-cache-'));
  const runtime = {
    cacheDir,
    timings: new Map(),
    recordTiming(kind, durationMs, cacheHit = false) {
      const current = this.timings.get(kind) || { count: 0, totalMs: 0, cacheHits: 0 };
      current.count += 1;
      current.totalMs += durationMs;
      if (cacheHit) current.cacheHits += 1;
      this.timings.set(kind, current);
    },
  };
  let computes = 0;

  const first = await runCachedJsonJob(runtime, {
    kind: 'unit',
    cacheKey: { style: 'apa', fixture: 'core', hash: 'a' },
    async compute() {
      computes += 1;
      return { value: 'first' };
    },
  });
  const second = await runCachedJsonJob(runtime, {
    kind: 'unit',
    cacheKey: { style: 'apa', fixture: 'core', hash: 'a' },
    async compute() {
      computes += 1;
      return { value: 'second' };
    },
  });
  const third = await runCachedJsonJob(runtime, {
    kind: 'unit',
    cacheKey: { style: 'apa', fixture: 'core', hash: 'b' },
    async compute() {
      computes += 1;
      return { value: 'third' };
    },
  });

  fs.rmSync(cacheDir, { recursive: true, force: true });
  assert.deepEqual(first, { value: 'first' });
  assert.deepEqual(second, { value: 'first' });
  assert.deepEqual(third, { value: 'third' });
  assert.equal(computes, 2);
});

test('mapWithConcurrency preserves input ordering under parallel execution', async () => {
  const values = [40, 5, 20, 1];
  const results = await mapWithConcurrency(values, 2, async (delay, index) => {
    await new Promise((resolve) => setTimeout(resolve, delay));
    return `${index}:${delay}`;
  });

  assert.deepEqual(results, ['0:40', '1:5', '2:20', '3:1']);
});

test('preflightSnapshots reports missing citeproc snapshots for citeproc-backed styles', () => {
  const policy = loadVerificationPolicy();
  const stylesDir = fs.mkdtempSync(path.join(os.tmpdir(), 'report-preflight-'));
  fs.writeFileSync(path.join(stylesDir, 'definitely-missing-style.csl'), '<style></style>');
  const issues = preflightSnapshots(
    [
      {
        name: 'missing-style',
        sourceName: 'definitely-missing-style',
        format: 'author-date',
      },
    ],
    policy,
    stylesDir
  );

  fs.rmSync(stylesDir, { recursive: true, force: true });
  assert.equal(issues.length, 1);
  assert.equal(issues[0].status, 'missing');
  assert.equal(issues[0].style, 'missing-style');
});
