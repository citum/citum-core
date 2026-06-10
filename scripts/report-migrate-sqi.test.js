const test = require('node:test');
const assert = require('node:assert/strict');

const {
  evaluateMinimizationAcceptance,
  normalizedEqual,
  strictSectionEquivalent,
  rowFidelityStatus,
  combinedFidelity,
  aggregateFidelityHeadline,
} = require('./report-migrate-sqi');

function okRow(styleClass, passed, total) {
  return {
    style: `${styleClass}-style-${passed}`,
    styleClass,
    fidelity: {
      citations: { passed, total },
      bibliography: { passed, total },
      error: null,
    },
  };
}

test('rowFidelityStatus classifies failures as data, not crashes', () => {
  assert.equal(rowFidelityStatus(okRow('numeric', 18, 18)), 'ok');
  assert.equal(rowFidelityStatus({ style: 'x', error: { error: 'migrate_failed' } }), 'migrate_failed');
  assert.equal(
    rowFidelityStatus({ style: 'x', fidelity: { error: 'oracle_exec_failed' } }),
    'oracle_failed'
  );
  assert.equal(rowFidelityStatus({ style: 'x' }), 'fidelity_skipped');
  assert.equal(
    rowFidelityStatus({
      style: 'x',
      fidelity: { citations: { passed: 0, total: 0 }, bibliography: { passed: 0, total: 0 } },
    }),
    'oracle_empty'
  );
});

test('combinedFidelity merges citation and bibliography pass rates', () => {
  const row = {
    style: 'x',
    fidelity: {
      citations: { passed: 18, total: 18 },
      bibliography: { passed: 16, total: 32 },
    },
  };
  assert.equal(combinedFidelity(row), 34 / 50);
  assert.equal(combinedFidelity({ style: 'x', error: { error: 'migrate_failed' } }), null);
});

test('aggregateFidelityHeadline keeps failures in the denominator', () => {
  const rows = [
    okRow('author-date', 18, 18),
    okRow('author-date', 17, 18),
    okRow('numeric', 9, 18),
    { style: 'broken', styleClass: 'note', error: { error: 'migrate_failed', details: 'boom' } },
  ];
  const headline = aggregateFidelityHeadline(rows, 0.9);
  assert.equal(headline.threshold, 90);
  assert.equal(headline.overall.measured, 4);
  assert.equal(headline.overall.atThreshold, 2);
  assert.equal(headline.overall.shareAtThreshold, 50);
  assert.equal(headline.overall.statuses.migrate_failed, 1);
  assert.deepEqual(Object.keys(headline.perClass).sort(), ['author-date', 'note', 'numeric']);
  assert.equal(headline.perClass['author-date'].shareAtThreshold, 100);
  assert.equal(headline.perClass.note.combined, null);
});

test('aggregateFidelityHeadline returns null when fidelity was skipped', () => {
  assert.equal(aggregateFidelityHeadline([{ style: 'a' }, { style: 'b' }], 0.9), null);
});

test('normalizedEqual ignores markup but preserves semantic text differences', () => {
  assert.equal(normalizedEqual('<i>Title</i>.', 'Title'), true);
  assert.equal(normalizedEqual('(Smith, Lee, Kumar, & Zhou, 2021)', '(Smith et al, 2021)'), false);
});

test('strictSectionEquivalent rejects fuzzy APA citation drift', () => {
  assert.equal(
    strictSectionEquivalent({
      entries: [
        {
          oracle: '(John Smith, Lee, Kumar, & Zhou, 2021)',
          citum: '(Smith, Lee, Kumar, et al, 2021)',
          match: true,
        },
      ],
    }),
    false
  );
});

test('evaluateMinimizationAcceptance requires strict minimized oracle equivalence', () => {
  const baseOracle = {
    citations: { passed: 1 },
    bibliography: { passed: 1 },
  };
  const minOracle = {
    citations: {
      passed: 1,
      entries: [
        {
          oracle: '(John Smith, Lee, Kumar, & Zhou, 2021)',
          citum: '(Smith, Lee, Kumar, et al, 2021)',
          match: true,
        },
      ],
    },
    bibliography: {
      passed: 1,
      entries: [
        {
          oracle: 'Hawking, S. (1988). A Brief History of Time. New York: Bantam Dell Publishing Group',
          citum: 'Hawking, S. (1988). A Brief History of Time. Bantam Dell Publishing Group',
          match: true,
        },
      ],
    },
  };

  const acceptance = evaluateMinimizationAcceptance({
    baseOracle,
    minOracle,
    minLoc: 5,
    baseLoc: 5661,
  });

  assert.equal(acceptance.passCountsHold, true);
  assert.equal(acceptance.locImproves, true);
  assert.deepEqual(acceptance.strict, { citations: false, bibliography: false });
  assert.equal(acceptance.accepted, false);
});

test('evaluateMinimizationAcceptance accepts strict equivalent smaller output', () => {
  const minOracle = {
    citations: {
      passed: 1,
      entries: [{ oracle: '(Kuhn, 1962)', citum: '(Kuhn, 1962)', match: true }],
    },
    bibliography: {
      passed: 1,
      entries: [{ oracle: 'Kuhn, T. S. (1962). Title', citum: 'Kuhn, T. S. (1962). Title' }],
    },
  };

  const acceptance = evaluateMinimizationAcceptance({
    baseOracle: { citations: { passed: 1 }, bibliography: { passed: 1 } },
    minOracle,
    minLoc: 10,
    baseLoc: 100,
  });

  assert.equal(acceptance.accepted, true);
});
