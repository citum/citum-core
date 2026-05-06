const {
  aggregateByType,
  detectNameOrder,
  detectSuppressions,
  findConsensusOrdering,
  generateYaml,
  normalizeLocalizedPageLabelPrefix,
} = require('./template-inferrer');
const { strict: assert } = require('assert');

// Mock helpers
function makeName(family, given) {
    return [{ family, given }];
}

console.log('Running detectNameOrder tests...');

// Basic cases
assert.equal(detectNameOrder('Smith, John', makeName('Smith', 'John')), 'family-first', 'Basic family-first failed');
assert.equal(detectNameOrder('John Smith', makeName('Smith', 'John')), 'given-first', 'Basic given-first failed');
assert.equal(detectNameOrder('Smith, J.', makeName('Smith', 'John')), 'family-first', 'Initials family-first failed');
assert.equal(detectNameOrder('J. Smith', makeName('Smith', 'John')), 'given-first', 'Initials given-first failed');

// Initials without periods (boundary checks)
assert.equal(detectNameOrder('Smith J', makeName('Smith', 'John')), 'family-first', 'No-period initial family-first failed');
assert.equal(detectNameOrder('J Smith', makeName('Smith', 'John')), 'given-first', 'No-period initial given-first failed');

// Case insensitivity
assert.equal(detectNameOrder('smith, john', makeName('Smith', 'John')), 'family-first', 'Case insensitive failed');

// Literal names (should fail gracefully)
assert.equal(detectNameOrder('World Bank', [{ literal: 'World Bank' }]), null, 'Literal name should allow null');

// Missing parts
assert.equal(detectNameOrder('Smith', makeName('Smith', '')), null, 'Missing given name should be null');

// Initial matching window
// "Thomas, B." where given is "Brian"
assert.equal(detectNameOrder('Thomas, B.', makeName('Thomas', 'Brian')), 'family-first', 'Initial matching failed');

// Complex text (simulating window extraction)
const text1 = 'In K. A. Ericsson, N. Charness';
const names1 = makeName('Ericsson', 'K. Anders');
assert.equal(detectNameOrder(text1, names1), 'given-first', 'Complex editor window failed');

const text2 = 'Ericsson, K. A., Charness, N.';
assert.equal(detectNameOrder(text2, names1), 'family-first', 'Complex author window failed');

console.log('All detectNameOrder tests passed!');

console.log('Running aggregateByType/detectSuppressions tests...');

const duplicateEntries = ['Shared Citation', 'Shared Citation'];
const duplicateRefs = [
    {
        type: 'article-journal',
        title: 'Shared Citation',
    },
    {
        type: 'article-journal',
        title: 'Shared Citation',
        issue: '7',
    },
];
const typedComponents = aggregateByType(duplicateEntries, duplicateRefs);
assert.deepEqual(
    typedComponents['article-journal'].entries.map(({ index, rendered }) => ({ index, rendered })),
    [
        { index: 0, rendered: 'Shared Citation' },
        { index: 1, rendered: 'Shared Citation' },
    ],
    'aggregateByType should retain original entry indices for duplicate rendered entries',
);
const suppressions = detectSuppressions(
    ['issue'],
    typedComponents,
    {},
    { entries: duplicateEntries },
    duplicateRefs,
);
assert.equal(
    suppressions.issue['article-journal'],
    true,
    'detectSuppressions should use the original entry index for duplicate rendered entries',
);

console.log('Running findConsensusOrdering tests...');

const orderingEntries = [
    'Smith Alpha Title (2020).',
    'Jones Beta Title (2021).',
];
const orderingRefs = [
    {
        type: 'book',
        author: makeName('Smith', 'John'),
        title: 'Alpha Title',
        issued: { 'date-parts': [[2020]] },
    },
    {
        type: 'book',
        author: makeName('Jones', 'Jane'),
        title: 'Beta Title',
        issued: { 'date-parts': [[2021]] },
    },
];
const ordering = findConsensusOrdering(orderingEntries, orderingRefs);
assert.deepEqual(
    ordering.consensusOrdering,
    ['contributors', 'title', 'year'],
    'findConsensusOrdering should use the provided refByEntry mapping directly',
);

console.log('Running localized page label inference tests...');

const normalizedPagePrefix = normalizeLocalizedPageLabelPrefix({ number: 'pages' }, ', pp. ');
assert.deepEqual(
    normalizedPagePrefix,
    { prefix: ', ', localized: true },
    'literal page label prefixes should be normalized to localized label semantics',
);

const pageYaml = generateYaml([
    { number: 'pages', prefix: ', ', 'label-form': 'short' },
]);
assert.equal(
    pageYaml,
    'template:\n  - number: pages\n    prefix: ", "\n    label-form: short\n',
    'generateYaml should emit label-form for localized page labels',
);

console.log('All template inferrer core tests passed!');
