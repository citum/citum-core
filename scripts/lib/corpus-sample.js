/*
 * Seeded, stratified sampling over the legacy CSL corpus.
 *
 * Used by report-migrate-sqi.js to build a reproducible "random" measurement
 * corpus: independent parent styles are classified by their CSL
 * `citation-format` category, then sampled per-stratum with a deterministic
 * PRNG so a given seed always yields the same corpus.
 */

'use strict';

const KNOWN_CLASSES = ['author-date', 'numeric', 'note', 'label', 'author'];

/** Deterministic 32-bit PRNG (mulberry32). Returns floats in [0, 1). */
function mulberry32(seed) {
  let a = seed >>> 0;
  return function next() {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Classify a CSL style by its `citation-format` category attribute.
 * Returns one of KNOWN_CLASSES or 'unknown'.
 */
function classifyCitationFormat(cslText) {
  if (typeof cslText !== 'string') return 'unknown';
  const match = /citation-format="([^"]+)"/.exec(cslText);
  if (!match) return 'unknown';
  const value = match[1].trim();
  return KNOWN_CLASSES.includes(value) ? value : 'unknown';
}

/** In-place Fisher–Yates shuffle driven by the supplied PRNG. */
function shuffle(items, rand) {
  for (let i = items.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [items[i], items[j]] = [items[j], items[i]];
  }
  return items;
}

/**
 * Allocate `sampleSize` slots across strata proportionally to stratum size,
 * holding a floor of `minPerStratum` (capped at stratum size) for every
 * non-empty stratum. Deterministic: ties resolve by stratum name.
 *
 * @param {Map<string, number>} stratumSizes class -> population count
 * @returns {Map<string, number>} class -> allocated count
 */
function allocateStrata(stratumSizes, sampleSize, minPerStratum) {
  const names = [...stratumSizes.keys()].sort();
  const population = names.reduce((sum, name) => sum + stratumSizes.get(name), 0);
  const allocation = new Map();
  if (population <= sampleSize) {
    for (const name of names) allocation.set(name, stratumSizes.get(name));
    return allocation;
  }

  // Scale the floor down when stratum floors alone would exceed the sample
  // size (e.g. a pilot draw of 10 across 5 strata), so the requested size
  // is always honored.
  const effectiveMin = Math.min(
    minPerStratum,
    Math.max(1, Math.floor(sampleSize / names.length))
  );
  for (const name of names) {
    const size = stratumSizes.get(name);
    const proportional = Math.floor((size / population) * sampleSize);
    allocation.set(name, Math.min(size, Math.max(proportional, effectiveMin)));
  }

  let total = [...allocation.values()].reduce((sum, n) => sum + n, 0);
  // Grow toward the target from the largest strata (most remaining capacity),
  // shrink from the largest allocations that are still above their floor.
  while (total < sampleSize) {
    const candidates = names
      .filter((name) => allocation.get(name) < stratumSizes.get(name))
      .sort((a, b) => stratumSizes.get(b) - stratumSizes.get(a) || a.localeCompare(b));
    if (candidates.length === 0) break;
    allocation.set(candidates[0], allocation.get(candidates[0]) + 1);
    total += 1;
  }
  while (total > sampleSize) {
    const floor = (name) => Math.min(stratumSizes.get(name), effectiveMin);
    const candidates = names
      .filter((name) => allocation.get(name) > floor(name))
      .sort((a, b) => allocation.get(b) - allocation.get(a) || a.localeCompare(b));
    if (candidates.length === 0) break;
    allocation.set(candidates[0], allocation.get(candidates[0]) - 1);
    total -= 1;
  }
  return allocation;
}

/**
 * Draw a seeded, stratified sample from a classified corpus.
 *
 * @param {Array<{style: string, styleClass: string}>} classified
 * @param {{sampleSize: number, seed: number, minPerStratum?: number}} options
 * @returns {{sample: Array<{style: string, styleClass: string}>,
 *            allocation: Object<string, number>,
 *            population: number,
 *            strata: Object<string, number>}}
 */
function stratifiedSample(classified, { sampleSize, seed, minPerStratum = 5 }) {
  const byClass = new Map();
  for (const entry of classified) {
    if (!byClass.has(entry.styleClass)) byClass.set(entry.styleClass, []);
    byClass.get(entry.styleClass).push(entry.style);
  }
  const stratumSizes = new Map();
  for (const [name, styles] of byClass) {
    styles.sort();
    stratumSizes.set(name, styles.length);
  }

  const allocation = allocateStrata(stratumSizes, sampleSize, minPerStratum);
  const rand = mulberry32(seed);
  const sample = [];
  // Consume the PRNG stream in sorted-stratum order so the draw is fully
  // determined by (seed, corpus contents), not map insertion order.
  for (const name of [...byClass.keys()].sort()) {
    const pool = shuffle([...byClass.get(name)], rand);
    for (const style of pool.slice(0, allocation.get(name) ?? 0)) {
      sample.push({ style, styleClass: name });
    }
  }
  sample.sort((a, b) => a.styleClass.localeCompare(b.styleClass) || a.style.localeCompare(b.style));

  return {
    sample,
    allocation: Object.fromEntries([...allocation.entries()].sort()),
    population: classified.length,
    strata: Object.fromEntries([...stratumSizes.entries()].sort()),
  };
}

module.exports = {
  KNOWN_CLASSES,
  mulberry32,
  classifyCitationFormat,
  allocateStrata,
  stratifiedSample,
};
