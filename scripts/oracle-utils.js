/**
 * Shared utilities for oracle scripts (oracle.js and oracle-yaml.js).
 *
 * Extracts common functions to enable consistent component parsing,
 * text normalization, and reference data lookup across oracle variants.
 */

'use strict';

const fs = require('fs');
const path = require('path');

// -- Text normalization --

/**
 * Normalize text by stripping HTML, markdown formatting, and
 * collapsing whitespace.
 */
function normalizeText(text) {
  const monthMap = {
    January: 'Jan',
    February: 'Feb',
    March: 'Mar',
    April: 'Apr',
    May: 'May',
    June: 'Jun',
    July: 'Jul',
    August: 'Aug',
    September: 'Sep',
    October: 'Oct',
    November: 'Nov',
    December: 'Dec',
  };

  return text
    .replace(/<[^>]+>/g, '')           // Strip HTML tags
    .replace(/&#38;/g, '&')            // HTML entity for &
    .replace(/_([^_]+)_/g, '$1')       // Strip markdown italics
    .replace(/\*\*([^*]+)\*\*/g, '$1') // Strip markdown bold
    .replace(/\[(Internet)\]/gi, '')   // Normalize optional medium marker
    .replace(/\bsec\.?\b/gi, 'section')
    .replace(/\((eds?|ed)\.\)/gi, 'editors')
    .replace(/\b(eds?|ed)\.?\b/gi, 'editors')
    .replace(
      /\[(cited)\s+(\d{4}),?\s+([A-Za-z]+)\s+(\d{1,2})\]/gi,
      (_, a, y, m, d) => `[${a} ${y} ${m} ${d}]`
    )
    .replace(
      /(Accessed)\s+(\d{4}),?\s+([A-Za-z]+)\s+(\d{1,2})/gi,
      (_, a, y, m, d) => `${a} ${d} ${m} ${y}`
    )
    .replace(
      /\b(January|February|March|April|May|June|July|August|September|October|November|December)\b/g,
      m => monthMap[m] || m
    )
    .replace(/\bet al\./gi, 'et al')   // Normalize equivalent et-al punctuation
    .replace(/;\./g, ';')
    .replace(/\s+([,.;:])/g, '$1')     // Normalize stray spaces before punctuation
    .replace(/\s+/g, ' ')             // Normalize whitespace
    // Strip bibliography numbering prefix after whitespace normalization (allow hidden directional marks).
    // Only strip when the numeric label is followed by actual entry content.
    .replace(/^[\u200e\u200f\u202a-\u202e\u2066-\u2069]*\d+\.(?=[^\d])\s*/, '')
    .replace(/[.,;:]\s*$/g, '')
    .trim();
}

/**
 * Tokenize normalized text for order/punctuation-tolerant similarity checks.
 *
 * Similarity intentionally remains case-insensitive so punctuation/order
 * regressions can still be matched after exact case-aware equality fails.
 *
 * @param {string} text
 * @returns {string[]}
 */
function tokenizeForSimilarity(text) {
  return normalizeText(text || '')
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s]/gu, ' ')
    .split(/\s+/)
    .filter(Boolean)
    .filter((token) => token.length > 1);
}

/**
 * Compute bag-of-words similarity for normalized text.
 *
 * @param {string} leftText
 * @param {string} rightText
 * @returns {number}
 */
function textSimilarity(leftText, rightText) {
  const left = tokenizeForSimilarity(leftText);
  const right = tokenizeForSimilarity(rightText);
  if (left.length === 0 && right.length === 0) return 1;
  if (left.length === 0 || right.length === 0) return 0;

  const leftCounts = new Map();
  const rightCounts = new Map();
  for (const token of left) {
    leftCounts.set(token, (leftCounts.get(token) || 0) + 1);
  }
  for (const token of right) {
    rightCounts.set(token, (rightCounts.get(token) || 0) + 1);
  }

  let intersect = 0;
  let union = 0;
  const keys = new Set([...leftCounts.keys(), ...rightCounts.keys()]);
  for (const key of keys) {
    const leftCount = leftCounts.get(key) || 0;
    const rightCount = rightCounts.get(key) || 0;
    intersect += Math.min(leftCount, rightCount);
    union += Math.max(leftCount, rightCount);
  }

  return union > 0 ? intersect / union : 0;
}

/**
 * Return true when texts differ only by letter case after normalization.
 *
 * @param {string} leftText
 * @param {string} rightText
 * @returns {boolean}
 */
function isCaseOnlyMismatch(leftText, rightText) {
  const left = normalizeText(leftText);
  const right = normalizeText(rightText);
  return left !== right && left.toLowerCase() === right.toLowerCase();
}

/**
 * Compare texts with exact case-aware equality first, then similarity fallback.
 *
 * @param {string} expectedText
 * @param {string} actualText
 * @param {{ caseSensitive?: boolean, similarityThreshold?: number }} [options]
 * @returns {{ expected: string, actual: string, match: boolean, caseMismatch: boolean, similarity: number }}
 */
function compareText(expectedText, actualText, options = {}) {
  const caseSensitive = options.caseSensitive !== false;
  const similarityThreshold = options.similarityThreshold ?? 0.60;
  const expected = normalizeText(expectedText);
  const actual = normalizeText(actualText);

  if (expected === actual) {
    return { expected, actual, match: true, caseMismatch: false, similarity: 1 };
  }

  const caseMismatch = expected.toLowerCase() === actual.toLowerCase();
  if (caseSensitive && caseMismatch) {
    return { expected, actual, match: false, caseMismatch: true, similarity: 1 };
  }

  const similarity = textSimilarity(expected, actual);
  return {
    expected,
    actual,
    match: similarity >= similarityThreshold,
    caseMismatch,
    similarity,
  };
}

// -- Reference data lookup --

/**
 * Find the reference data matching a bibliography entry.
 *
 * Uses multi-field scoring (author + year + title) to disambiguate
 * entries that share an author name.
 *
 * @param {string} entry - The bibliography entry text
 * @param {Object} testItems - Map of item IDs to reference data
 * @returns {{ id: string, ref: Object }|null} The best-matching reference and id, or null
 */
function hasPrimaryNames(ref) {
  return ['author', 'editor', 'translator'].some(
    (role) => Array.isArray(ref?.[role]) && ref[role].length > 0
  );
}

function findRefMatchForEntry(entry, testItems) {
  const entryLower = normalizeText(entry).toLowerCase();
  const candidates = [];

  for (const [id, ref] of Object.entries(testItems)) {
    let score = 0;
    let nameMatch = false;

    // Check author family name
    if (ref.author && ref.author.length > 0) {
      const first = ref.author[0];
      const name = (first.family || first.literal || '').toLowerCase();
      if (name && entryLower.includes(name)) {
        score += 1;
        nameMatch = true;
      }
    }

    // Check editor if no author match
    if (!nameMatch && ref.editor && ref.editor.length > 0) {
      const first = ref.editor[0];
      const name = (first.family || first.literal || '').toLowerCase();
      if (name && entryLower.includes(name)) {
        score += 1;
        nameMatch = true;
      }
    }

    if (!nameMatch) {
      if (!hasPrimaryNames(ref) && ref.title) {
        const titleLower = normalizeText(ref.title).toLowerCase();
        if (entryLower.includes(titleLower)) {
          score += 4;
        } else {
          const prefix = titleLower.substring(0, Math.min(40, titleLower.length));
          if (prefix.length >= 10 && entryLower.includes(prefix)) {
            score += 2;
          }
        }
      }
      if (score === 0) continue;
    }

    // Year match (strongly discriminating)
    if (ref.issued && ref.issued['date-parts'] && ref.issued['date-parts'][0]) {
      const year = String(ref.issued['date-parts'][0][0]);
      if (entryLower.includes(year)) {
        score += 2;
      }
    }

    // Title match (full title is the strongest signal)
    if (ref.title) {
      const titleLower = normalizeText(ref.title).toLowerCase();
      if (entryLower.includes(titleLower)) {
        score += 4;
      } else {
        // Partial title: use first 40 chars (styles may sentence-case or truncate)
        const prefix = titleLower.substring(0, Math.min(40, titleLower.length));
        if (prefix.length >= 10 && entryLower.includes(prefix)) {
          score += 2;
        }
      }
    }

    candidates.push({ id, ref, score });
  }

  if (candidates.length === 0) return null;

  // Return highest-scoring candidate
  candidates.sort((a, b) => b.score - a.score);
  return candidates[0];
}

function findRefDataForEntry(entry, testItems) {
  return findRefMatchForEntry(entry, testItems)?.ref || null;
}

// -- Position helpers --

function escapeRegex(str) {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Find the position of a string field value in the entry.
 * Uses case-insensitive full-field matching.
 *
 * @returns {{ start: number, end: number }|null}
 */
function findFieldPosition(entryLower, fieldValue) {
  if (!fieldValue) return null;
  const normalized = normalizeText(String(fieldValue)).toLowerCase();
  if (!normalized) return null;

  const idx = entryLower.indexOf(normalized);
  if (idx >= 0) {
    return { start: idx, end: idx + normalized.length };
  }
  return null;
}

/**
 * Find a numeric field value with digit-boundary guards to prevent
 * false positives. For example, volume "10" must not match "2010".
 *
 * @returns {{ start: number, end: number }|null}
 */
function findNumericFieldPosition(entryLower, value) {
  if (value == null) return null;
  const str = String(value);
  // Digit-boundary: not preceded or followed by another digit
  const regex = new RegExp(`(?<![0-9])${escapeRegex(str)}(?![0-9])`);
  const match = entryLower.match(regex);
  if (match) {
    return { start: match.index, end: match.index + str.length };
  }
  return null;
}

/**
 * Expand a name component position to include given name or initials if nearby.
 * @param {string} entryLower - Normalized lower-case entry text
 * @param {Object} familyPos - {start, end} of the family name
 * @param {string} givenName - The given name to search for
 */
function expandNamePosition(entryLower, familyPos, givenName) {
  if (!familyPos) return null;

  if (!givenName) return familyPos;

  const givenLower = givenName.toLowerCase();
  const givenInitial = givenName.charAt(0).toLowerCase();

  // Try finding full given name or initial near the family name
  const searchWindow = 50;
  const start = Math.max(0, familyPos.start - searchWindow);
  const end = Math.min(entryLower.length, familyPos.end + searchWindow);
  const text = entryLower.substring(start, end);

  let givenPos = null;
  let givenLen = 0;

  // 1. Try full given name
  const idxFull = text.indexOf(givenLower);
  if (idxFull !== -1) {
    givenPos = start + idxFull;
    givenLen = givenLower.length;
  } else {
    // 2. Try initial with word boundary
    const initialRegex = new RegExp(`\\b${escapeRegex(givenInitial)}\\.?`, 'i');
    const match = text.match(initialRegex);
    if (match) {
      givenPos = start + match.index;
      givenLen = match[0].length;
    }
  }

  if (givenPos !== null) {
    // Merge ranges
    const mergedStart = Math.min(familyPos.start, givenPos);
    const mergedEnd = Math.max(familyPos.end, givenPos + givenLen);
    return { start: mergedStart, end: mergedEnd };
  }

  return familyPos;
}

// -- Component parsing --

/**
 * Parse rendered bibliography entry into semantic components.
 *
 * @param {string} entry - The bibliography entry text
 * @param {Object} refData - The reference data (from JSON fixture)
 * @returns {Object} Components found: { contributors, year, title, ... }
 */
function parseComponents(entry, refData) {
  const result = {
    contributors: { found: false, value: null, pos: null },
    year: { found: false, value: null, pos: null },
    title: { found: false, value: null, pos: null },
    containerTitle: { found: false, value: null, pos: null },
    volume: { found: false, value: null, pos: null },
    issue: { found: false, value: null, pos: null },
    pages: { found: false, value: null, pos: null },
    publisher: { found: false, value: null, pos: null },
    doi: { found: false, value: null, pos: null },
    edition: { found: false, value: null, pos: null },
    editors: { found: false, value: null, pos: null }
  };

  const entryLower = normalizeText(entry).toLowerCase();

  // Contributors (authors/editors)
  if (refData.author && refData.author.length > 0) {
    const first = refData.author[0];
    const familyName = (first.family || first.literal || '').toLowerCase();
    if (familyName) {
      const familyPos = findFieldPosition(entryLower, familyName);
      const fullPos = expandNamePosition(entryLower, familyPos, first.given);
      if (fullPos) {
        result.contributors = {
          found: true,
          value: entry.substring(fullPos.start, fullPos.end),
          pos: fullPos
        };
      }
    }
  }

  // Year
  if (refData.issued && refData.issued['date-parts'] && refData.issued['date-parts'][0]) {
    const year = String(refData.issued['date-parts'][0][0]);
    const yearPos = findNumericFieldPosition(entryLower, year);
    if (yearPos) {
      result.year = {
        found: true,
        value: entry.substring(yearPos.start, yearPos.end),
        pos: yearPos
      };
    }
  }

  // Title
  if (refData.title) {
    const titlePos = findFieldPosition(entryLower, refData.title);
    if (titlePos) {
      result.title = {
        found: true,
        value: entry.substring(titlePos.start, titlePos.end),
        pos: titlePos
      };
    }
  }

  // Container title (journal, book series, etc.)
  if (refData['container-title']) {
    const containerPos = findFieldPosition(entryLower, refData['container-title']);
    if (containerPos) {
      result.containerTitle = {
        found: true,
        value: entry.substring(containerPos.start, containerPos.end),
        pos: containerPos
      };
    }
  }

  // Volume
  if (refData.volume != null) {
    const volPos = findNumericFieldPosition(entryLower, refData.volume);
    if (volPos) {
      result.volume = {
        found: true,
        value: entry.substring(volPos.start, volPos.end),
        pos: volPos
      };
    }
  }

  // Issue
  if (refData.issue != null) {
    const issuePos = findNumericFieldPosition(entryLower, refData.issue);
    if (issuePos) {
      result.issue = {
        found: true,
        value: entry.substring(issuePos.start, issuePos.end),
        pos: issuePos
      };
    }
  }

  // Pages
  if (refData.page) {
    const pagePos = findFieldPosition(entryLower, refData.page);
    if (pagePos) {
      result.pages = {
        found: true,
        value: entry.substring(pagePos.start, pagePos.end),
        pos: pagePos
      };
    }
  }

  // Publisher
  if (refData.publisher) {
    const pubPos = findFieldPosition(entryLower, refData.publisher);
    if (pubPos) {
      result.publisher = {
        found: true,
        value: entry.substring(pubPos.start, pubPos.end),
        pos: pubPos
      };
    }
  }

  // DOI
  if (refData.DOI) {
    const doiPos = findFieldPosition(entryLower, refData.DOI);
    if (doiPos) {
      result.doi = {
        found: true,
        value: entry.substring(doiPos.start, doiPos.end),
        pos: doiPos
      };
    }
  }

  // Edition
  if (refData.edition != null) {
    const edPos = findFieldPosition(entryLower, String(refData.edition));
    if (edPos) {
      result.edition = {
        found: true,
        value: entry.substring(edPos.start, edPos.end),
        pos: edPos
      };
    }
  }

  // Editors
  if (refData.editor && refData.editor.length > 0) {
    const first = refData.editor[0];
    const familyName = (first.family || first.literal || '').toLowerCase();
    if (familyName) {
      const familyPos = findFieldPosition(entryLower, familyName);
      const fullPos = expandNamePosition(entryLower, familyPos, first.given);
      if (fullPos) {
        result.editors = {
          found: true,
          value: entry.substring(fullPos.start, fullPos.end),
          pos: fullPos
        };
      }
    }
  }

  return result;
}

/**
 * Analyze component ordering in a bibliography entry.
 *
 * @param {Object} components - Parsed components from parseComponents()
 * @returns {Array<string>} Ordered list of component names by position
 */
function analyzeOrdering(components) {
  const positioned = Object.entries(components)
    .filter(([_, data]) => data.found && data.pos)
    .map(([name, data]) => ({ name, pos: data.pos }))
    .sort((a, b) => a.pos.start - b.pos.start);

  return positioned.map(item => item.name);
}

/**
 * Load a locale XML file.
 * @param {string} lang - Language code (e.g., 'en-US')
 * @returns {string} XML content
 */
function loadLocale(lang) {
  const localePath = path.join(__dirname, `locales-${lang}.xml`);
  if (fs.existsSync(localePath)) {
    return fs.readFileSync(localePath, 'utf8');
  }
  const fallback = path.join(__dirname, 'locales-en-US.xml');
  if (fs.existsSync(fallback)) {
    return fs.readFileSync(fallback, 'utf8');
  }
  throw new Error(`Locale not found: ${lang}`);
}

module.exports = {
  compareText,
  normalizeText,
  isCaseOnlyMismatch,
  parseComponents,
  analyzeOrdering,
  findRefMatchForEntry,
  hasPrimaryNames,
  findRefDataForEntry,
  loadLocale,
  findFieldPosition,
  findNumericFieldPosition,
  expandNamePosition,
  escapeRegex,
  textSimilarity,
  tokenizeForSimilarity,
};
