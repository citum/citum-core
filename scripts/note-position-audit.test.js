const test = require('node:test');
const assert = require('node:assert/strict');

const {
  evaluateConformanceLayer,
  evaluateNotePositionRender,
  evaluateRegressionLayer,
  normalizeFixtureLocators,
  parseRenderedCitations,
  summarizeAuditResults,
  validateExpectations,
  validateExpectationCoverage,
} = require('./lib/note-position-audit');

const EXPECTATIONS = validateExpectations({
  version: 2,
  regression_profiles: {
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
  conformance_families: {
    'chicago-full-note': {
      lexical_marker: 'ibid',
      immediate_repeat_form: 'marker',
      immediate_with_locator_form: 'marker',
      shortened_note_source: 'subsequent',
      distinct_subsequent: true,
      unresolved: ['note-start', 'prose-integral'],
    },
    'mhra-full-note': {
      lexical_marker: 'none',
      immediate_repeat_form: 'shortened-note',
      immediate_with_locator_form: 'shortened-note',
      shortened_note_source: 'subsequent',
      distinct_subsequent: true,
      unresolved: ['note-start', 'prose-integral'],
    },
  },
  styles: {
    'example-ibid': {
      regression_profile: 'ibid-and-subsequent',
      conformance_family: 'chicago-full-note',
    },
    'example-fallback': {
      regression_profile: 'subsequent-fallback',
      conformance_family: 'mhra-full-note',
    },
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

test('evaluateRegressionLayer passes for lexical ibid and distinct subsequent', () => {
  const result = evaluateRegressionLayer(
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

test('evaluateRegressionLayer reports configuration gaps for missing subsequent override', () => {
  const result = evaluateRegressionLayer(
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

test('evaluateConformanceLayer reports unresolved dimensions without failing a settled match', () => {
  const result = evaluateConformanceLayer(
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
  assert.deepEqual(result.unresolved, ['note-start', 'prose-integral']);
});

test('evaluateNotePositionRender preserves regression aliases while exposing layered results', () => {
  const result = evaluateNotePositionRender(
    'example-fallback',
    { citation: { subsequent: {} } },
    {
      'note-first': 'John Smith, Full Cite',
      'note-ibid': 'Smith, Short Cite',
      'note-ibid-with-locator': 'Smith, Short Cite, 105',
      'note-intervening': 'Other Author, Other Cite',
      'note-subsequent': 'Smith, Short Cite',
      'note-subsequent-with-locator': 'Smith, Short Cite, 205',
    },
    EXPECTATIONS
  );

  assert.equal(result.status, 'pass');
  assert.equal(result.profile, 'subsequent-fallback');
  assert.equal(result.regression.status, 'pass');
  assert.equal(result.conformance.status, 'pass');
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

test('summarizeAuditResults counts each layer separately', () => {
  const summary = summarizeAuditResults(
    [
      {
        regression: { status: 'pass' },
        conformance: { status: 'pass', unresolved: ['note-start'] },
      },
      {
        regression: { status: 'configuration-gap' },
        conformance: { status: 'gap', unresolved: [] },
      },
      {
        regression: { status: 'rendering-gap' },
        conformance: { status: 'pass', unresolved: [] },
      },
    ],
    { missing: ['style-a'], extra: [] }
  );

  assert.deepEqual(summary, {
    total: 3,
    regression: {
      pass: 1,
      configurationGap: 1,
      renderingGap: 1,
    },
    conformance: {
      pass: 2,
      gap: 1,
      unresolved: 1,
    },
    missingExpectations: 1,
    extraExpectations: 0,
  });
});
