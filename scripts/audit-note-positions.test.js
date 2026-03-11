const test = require('node:test');
const assert = require('node:assert/strict');

const { parseArgs } = require('./audit-note-positions');

test('audit-note-positions parses --styles list', () => {
  assert.deepEqual(parseArgs(['--json', '--styles', 'chicago-notes,oscola']), {
    jsonOutput: true,
    styles: ['chicago-notes', 'oscola'],
  });
});

test('audit-note-positions rejects missing --styles value', () => {
  assert.throws(
    () => parseArgs(['--styles']),
    /--styles option requires a comma-separated list of styles/
  );
});
