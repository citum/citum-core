const test = require('node:test');
const assert = require('node:assert/strict');

const {
  evaluateNotePositionRender,
  normalizeFixtureLocators,
  parseRenderedCitations,
  summarizeAuditResults,
  validateExpectations,
  validateExpectationCoverage,
} = require('./lib/note-position-audit');

const EXPECTATIONS = validateExpectations({
  version: 1,
  profiles: {
    'ibid-and-subsequent': {
      requires: { ibid: true, subsequent: true },
      checks: {
        lexical_ibid: true,
        immediate_falls_back_to_subsequent: false,
        distinct_subsequent: true,
      },
    },
    'subsequent-fallback': {
      requires: { ibid: false, subsequent: true },
      checks: {
        lexical_ibid: false,
        immediate_falls_back_to_subsequent: true,
        distinct_subsequent: true,
      },
    },
  },
  styles: {
    'example-ibid': { profile: 'ibid-and-subsequent' },
    'example-fallback': { profile: 'subsequent-fallback' },
  },
});

test('parseRenderedCitations extracts keyed citation output', () => {
  const output = `
=== demo ===

CITATIONS (From file):
  [note-first] First cite
  [note-ibid] Ibid.

`;

  assert.deepEqual(parseRenderedCitations(output), {
    'note-first': 'First cite',
    'note-ibid': 'Ibid.',
  });
});

test('evaluateNotePositionRender passes for lexical ibid and distinct subsequent', () => {
  const result = evaluateNotePositionRender(
    'example-ibid',
    { citation: { ibid: {}, subsequent: {} } },
    {
      'note-first': 'John Smith, Full Cite',
      'note-ibid': 'Ibid.',
      'note-ibid-with-locator': 'Ibid., 105',
      'note-intervening': 'Other Author, Other Cite',
      'note-subsequent': 'Smith, Short Cite',
      'note-subsequent-with-locator': 'Smith, Short Cite, 205',
    },
    EXPECTATIONS
  );

  assert.equal(result.status, 'pass');
  assert.deepEqual(result.issues, []);
});

test('evaluateNotePositionRender reports configuration gaps for missing subsequent override', () => {
  const result = evaluateNotePositionRender(
    'example-fallback',
    { citation: {} },
    {
      'note-first': 'John Smith, Full Cite',
      'note-ibid': 'Smith, Short Cite',
      'note-ibid-with-locator': 'Smith, Short Cite, 205',
      'note-intervening': 'Other Author, Other Cite',
      'note-subsequent': 'Smith, Short Cite',
      'note-subsequent-with-locator': 'Smith, Short Cite, 205',
    },
    EXPECTATIONS
  );

  assert.equal(result.status, 'configuration-gap');
  assert.match(result.issues[0].message, /citation\.subsequent/);
});

test('normalizeFixtureLocators ignores fixture-specific page numbers', () => {
  assert.equal(
    normalizeFixtureLocators('Smith, Short Cite, 105'),
    normalizeFixtureLocators('Smith, Short Cite, 205')
  );
});

test('validateExpectationCoverage finds missing and extra style declarations', () => {
  const coverage = validateExpectationCoverage(
    [{ name: 'example-ibid' }, { name: 'missing-style' }],
    EXPECTATIONS
  );

  assert.deepEqual(coverage, {
    missing: ['missing-style'],
    extra: ['example-fallback'],
  });
});

test('summarizeAuditResults counts each gap class', () => {
  const summary = summarizeAuditResults(
    [
      { status: 'pass' },
      { status: 'configuration-gap' },
      { status: 'rendering-gap' },
    ],
    { missing: ['style-a'], extra: [] }
  );

  assert.deepEqual(summary, {
    total: 3,
    pass: 1,
    configurationGap: 1,
    renderingGap: 1,
    missingExpectations: 1,
    extraExpectations: 0,
  });
});
