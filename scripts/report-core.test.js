const test = require('node:test');
const assert = require('node:assert/strict');

const {
  loadVerificationPolicy,
  resolveVerificationPolicy,
} = require('./lib/verification-policy');
const {
  discoverCoreStyles,
  formatAuthorityLabel,
  getComparisonEntryTexts,
  selectPrimaryComparator,
} = require('./report-core');

function loadStyleMap() {
  return new Map(discoverCoreStyles().map((style) => [style.name, style]));
}

test('discoverCoreStyles classifies representative style origins and CSL reach', () => {
  const styles = loadStyleMap();

  assert.equal(styles.get('apa-7th').originLabel, 'CSL migrated');
  assert.equal(styles.get('apa-7th').cslReach, 783);

  assert.equal(styles.get('chem-acs').originLabel, 'CSL hand-authored');
  assert.equal(styles.get('chem-acs').cslReach, null);

  assert.equal(styles.get('numeric-comp').originLabel, 'biblatex hand-authored');
  assert.equal(styles.get('numeric-comp').cslReach, null);

  const unknownOrigins = [...styles.values()].filter((style) => style.originLabel === 'Unknown');
  assert.deepEqual(unknownOrigins, []);
});

test('report-core exposes expected benchmark labels for representative styles', () => {
  const styles = loadStyleMap();
  const policy = loadVerificationPolicy();

  const cases = [
    ['apa-7th', 'citeproc-js'],
    ['chem-acs', 'Citum baseline'],
    ['numeric-comp', 'Citum baseline'],
  ];

  for (const [styleName, expectedLabel] of cases) {
    const style = styles.get(styleName);
    const stylePolicy = resolveVerificationPolicy(styleName, policy);
    const comparator = selectPrimaryComparator(style, stylePolicy);
    assert.equal(formatAuthorityLabel(comparator), expectedLabel);
  }
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
