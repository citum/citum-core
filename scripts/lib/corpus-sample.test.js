const test = require('node:test');
const assert = require('node:assert/strict');

const {
  mulberry32,
  classifyCitationFormat,
  allocateStrata,
  stratifiedSample,
} = require('./corpus-sample');

test('classifyCitationFormat reads the citation-format category attribute', () => {
  const csl = '<style class="in-text"><info><category citation-format="author-date"/></info></style>';
  assert.equal(classifyCitationFormat(csl), 'author-date');
  assert.equal(classifyCitationFormat('<style class="note"><info><category citation-format="note"/></info></style>'), 'note');
  assert.equal(classifyCitationFormat('<style><info/></style>'), 'unknown');
  assert.equal(classifyCitationFormat('<category citation-format="bogus-value"/>'), 'unknown');
  assert.equal(classifyCitationFormat(null), 'unknown');
});

test('mulberry32 is deterministic per seed and differs across seeds', () => {
  const first = mulberry32(42);
  const second = mulberry32(42);
  const sequenceA = [first(), first(), first()];
  const sequenceB = [second(), second(), second()];
  assert.deepEqual(sequenceA, sequenceB);
  const other = mulberry32(43);
  assert.notDeepEqual(sequenceA, [other(), other(), other()]);
  for (const value of sequenceA) {
    assert.ok(value >= 0 && value < 1);
  }
});

test('allocateStrata holds per-stratum floors and hits the sample size', () => {
  const sizes = new Map([
    ['author-date', 1000],
    ['numeric', 1500],
    ['note', 300],
    ['label', 8],
  ]);
  const allocation = allocateStrata(sizes, 100, 5);
  const total = [...allocation.values()].reduce((sum, n) => sum + n, 0);
  assert.equal(total, 100);
  assert.ok(allocation.get('label') >= 5);
  assert.ok(allocation.get('note') >= 5);
  assert.ok(allocation.get('numeric') > allocation.get('author-date'));
});

test('allocateStrata honors small sample sizes by scaling the floor', () => {
  const sizes = new Map([
    ['author', 10],
    ['author-date', 1335],
    ['label', 3],
    ['note', 542],
    ['numeric', 954],
  ]);
  const allocation = allocateStrata(sizes, 10, 5);
  const total = [...allocation.values()].reduce((sum, n) => sum + n, 0);
  assert.equal(total, 10);
  for (const count of allocation.values()) {
    assert.ok(count >= 1);
  }
});

test('allocateStrata takes everything when population fits the sample', () => {
  const sizes = new Map([['author-date', 3], ['note', 2]]);
  const allocation = allocateStrata(sizes, 100, 5);
  assert.deepEqual([...allocation.entries()].sort(), [
    ['author-date', 3],
    ['note', 2],
  ]);
});

function syntheticCorpus() {
  const classified = [];
  const push = (styleClass, count) => {
    for (let i = 0; i < count; i++) {
      classified.push({ style: `${styleClass}-style-${String(i).padStart(3, '0')}`, styleClass });
    }
  };
  push('author-date', 120);
  push('numeric', 180);
  push('note', 40);
  push('label', 6);
  return classified;
}

test('stratifiedSample is reproducible for a given seed', () => {
  const corpus = syntheticCorpus();
  const first = stratifiedSample(corpus, { sampleSize: 50, seed: 20260610 });
  const second = stratifiedSample(corpus, { sampleSize: 50, seed: 20260610 });
  assert.deepEqual(first.sample, second.sample);
  assert.equal(first.sample.length, 50);
  const different = stratifiedSample(corpus, { sampleSize: 50, seed: 7 });
  assert.notDeepEqual(
    first.sample.map((entry) => entry.style),
    different.sample.map((entry) => entry.style)
  );
});

test('stratifiedSample covers every non-empty stratum and reports metadata', () => {
  const corpus = syntheticCorpus();
  const drawn = stratifiedSample(corpus, { sampleSize: 50, seed: 1 });
  const sampledClasses = new Set(drawn.sample.map((entry) => entry.styleClass));
  assert.deepEqual([...sampledClasses].sort(), ['author-date', 'label', 'note', 'numeric']);
  assert.equal(drawn.population, corpus.length);
  assert.equal(drawn.strata.numeric, 180);
  const allocated = Object.values(drawn.allocation).reduce((sum, n) => sum + n, 0);
  assert.equal(allocated, 50);
  // label stratum (6 styles) is below the floor of 5 only if smaller than it;
  // here it must contribute at least min(5, 6) = 5 styles.
  assert.ok(drawn.allocation.label >= 5);
});

test('stratifiedSample draws without duplicates', () => {
  const drawn = stratifiedSample(syntheticCorpus(), { sampleSize: 100, seed: 99 });
  const names = drawn.sample.map((entry) => entry.style);
  assert.equal(new Set(names).size, names.length);
});
