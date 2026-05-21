const test = require('node:test');
const assert = require('node:assert/strict');

const {
  evaluateMinimizationAcceptance,
  normalizedEqual,
  strictSectionEquivalent,
} = require('./report-migrate-sqi');

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
