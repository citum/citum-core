const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');

const {
  DATASETS,
  formatDatasetMessage,
  maybeDatasetErrorForFile,
} = require('./lib/dataset-guard');

test('dataset guard message points developers to bootstrap full', () => {
  const message = formatDatasetMessage([DATASETS.legacyStyles], 'oracle.js');

  assert.match(message, /oracle\.js requires optional local datasets/);
  assert.match(message, /styles-legacy/);
  assert.match(message, /\.\/scripts\/bootstrap\.sh full/);
});

test('dataset guard recognizes missing files inside optional corpora', () => {
  const missingStyle = path.join(__dirname, '..', 'styles-legacy', 'definitely-missing-style.csl');
  const message = maybeDatasetErrorForFile(missingStyle, 'oracle.js');

  assert.equal(typeof message, 'string');
  assert.match(message, /legacy CSL styles corpus/);
});
