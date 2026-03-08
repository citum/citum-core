#!/usr/bin/env node

'use strict';

const fs = require('fs');
const path = require('path');

const PROJECT_ROOT = path.resolve(__dirname, '..', '..');
const BOOTSTRAP_COMMAND = './scripts/bootstrap.sh full';
const DATASETS = {
  legacyStyles: {
    key: 'legacyStyles',
    label: 'legacy CSL styles corpus',
    relativePath: 'styles-legacy',
  },
  cslTestSuite: {
    key: 'cslTestSuite',
    label: 'CSL test suite corpus',
    relativePath: path.join('tests', 'csl-test-suite'),
  },
};

function absoluteDatasetPath(dataset) {
  return path.join(PROJECT_ROOT, dataset.relativePath);
}

function datasetExists(key) {
  const dataset = DATASETS[key];
  if (!dataset) {
    throw new Error(`Unknown dataset key: ${key}`);
  }
  return fs.existsSync(absoluteDatasetPath(dataset));
}

function missingDatasets(keys) {
  return keys
    .map((key) => DATASETS[key])
    .filter(Boolean)
    .filter((dataset) => !fs.existsSync(absoluteDatasetPath(dataset)));
}

function formatDatasetMessage(datasets, context) {
  const summary = datasets.map((dataset) => `${dataset.label} (${dataset.relativePath})`).join(', ');
  return [
    `${context} requires optional local datasets that are not checked out: ${summary}.`,
    'Use the lean daily setup for normal Rust work, or fetch the corpora on demand with:',
    `  ${BOOTSTRAP_COMMAND}`,
  ].join('\n');
}

function ensureDatasets(keys, context) {
  const missing = missingDatasets(keys);
  if (missing.length > 0) {
    throw new Error(formatDatasetMessage(missing, context));
  }
}

function maybeDatasetErrorForFile(filePath, context) {
  const absolutePath = path.resolve(filePath);

  for (const dataset of Object.values(DATASETS)) {
    const datasetRoot = absoluteDatasetPath(dataset);
    if (absolutePath === datasetRoot || absolutePath.startsWith(`${datasetRoot}${path.sep}`)) {
      if (!fs.existsSync(absolutePath)) {
        return formatDatasetMessage([dataset], context);
      }
      return null;
    }
  }

  return null;
}

module.exports = {
  BOOTSTRAP_COMMAND,
  DATASETS,
  PROJECT_ROOT,
  datasetExists,
  ensureDatasets,
  formatDatasetMessage,
  maybeDatasetErrorForFile,
  missingDatasets,
};
