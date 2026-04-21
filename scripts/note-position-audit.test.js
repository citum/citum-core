const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {
  discoverNoteStyles,
  evaluateConformanceLayer,
  evaluateNotePositionRender,
  evaluateRegressionLayer,
  matchesLeadingTextCase,
  normalizeFixtureLocators,
  parseRenderedCitations,
  summarizeAuditResults,
  validateExpectations,
  validateExpectationCoverage,
} = require('./lib/note-position-audit');

const EXPECTATIONS = validateExpectations({
  version: 3,
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
      note_start_text_case: 'capitalize-first',
      immediate_repeat_form: 'marker',
      immediate_with_locator_form: 'marker',
      shortened_note_source: 'subsequent',
      distinct_subsequent: true,
      unresolved: ['prose-integral'],
    },
    oscola: {
      lexical_marker: 'ibid',
      note_start_text_case: 'lowercase',
      immediate_repeat_form: 'marker',
      immediate_with_locator_form: 'marker',
      shortened_note_source: 'subsequent',
      distinct_subsequent: true,
      unresolved: ['prose-integral'],
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
    'example-oscola': {
      regression_profile: 'ibid-and-subsequent',
      conformance_family: 'oscola',
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
    { citation: { ibid: { note_start_text_case: 'capitalize-first' }, subsequent: {} } },
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
  assert.deepEqual(result.unresolved, ['prose-integral']);
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

test('evaluateConformanceLayer enforces lowercase note-start ibid for oscola', () => {
  const result = evaluateConformanceLayer(
    'example-oscola',
    { citation: { ibid: { note_start_text_case: 'lowercase' }, subsequent: {} } },
    {
      'note-first': 'John Smith, Full Cite',
      'note-ibid': 'ibid.',
      'note-ibid-with-locator': 'ibid. 105',
      'note-intervening': 'Other Author, Other Cite',
      'note-subsequent': 'Smith, Short Cite',
      'note-subsequent-with-locator': 'Smith, Short Cite, 205',
    },
    EXPECTATIONS
  );

  assert.equal(result.status, 'pass');
  assert.deepEqual(result.unresolved, ['prose-integral']);
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
    extra: ['example-fallback', 'example-oscola'],
  });
});

test('discoverNoteStyles skips hidden embedded core wrappers', () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'note-style-discovery-'));
  const embeddedDir = path.join(fixtureRoot, 'embedded');
  fs.mkdirSync(embeddedDir, { recursive: true });

  fs.writeFileSync(
    path.join(embeddedDir, 'public-note.yaml'),
    'options:\n  processing: note\n',
    'utf8'
  );
  fs.writeFileSync(
    path.join(embeddedDir, 'public-note-core.yaml'),
    'options:\n  processing: note\n',
    'utf8'
  );

  assert.deepEqual(
    discoverNoteStyles(fixtureRoot).map((style) => style.name),
    ['public-note']
  );

  fs.rmSync(fixtureRoot, { recursive: true, force: true });
});

test('summarizeAuditResults counts each layer separately', () => {
  const summary = summarizeAuditResults(
    [
      {
        regression: { status: 'pass' },
        conformance: { status: 'pass', unresolved: ['prose-integral'] },
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

test('matchesLeadingTextCase detects capitalized and lowercase note-start markers', () => {
  assert.equal(matchesLeadingTextCase('Ibid., 105', 'capitalize-first'), true);
  assert.equal(matchesLeadingTextCase('ibid. 105', 'lowercase'), true);
  assert.equal(matchesLeadingTextCase('Ibid., 105', 'lowercase'), false);
});
