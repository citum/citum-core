'use strict';

const {
  compareText,
  findRefMatchForEntry,
  hasPrimaryNames,
  normalizeText,
} = require('../oracle-utils');
const {
  loadVerificationPolicy,
  resolveRegisteredDivergence,
} = require('./verification-policy');

const DIV_004_ID = 'div-004';
const DIV_005_ID = 'div-005';

function escapeRegex(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function arraysEqual(left, right) {
  if (left.length !== right.length) return false;
  return left.every((value, index) => value === right[index]);
}

function buildReferenceOrderIds(entries, testItems) {
  return entries
    .map((entry) => findRefMatchForEntry(entry, testItems)?.id || null)
    .filter(Boolean);
}

function buildNumericLabelMap(orderIds) {
  return new Map(orderIds.map((id, index) => [id, index + 1]));
}

function detectDiv004OrderDifference(oracleBibliography, citumOrderIds, testItems, divergenceRule) {
  if (!divergenceRule || !Array.isArray(oracleBibliography) || !Array.isArray(citumOrderIds)) {
    return null;
  }

  const oracleOrderIds = buildReferenceOrderIds(oracleBibliography, testItems);
  if (oracleOrderIds.length === 0 || oracleOrderIds.length !== citumOrderIds.length) {
    return null;
  }

  const oracleSet = new Set(oracleOrderIds);
  const citumSet = new Set(citumOrderIds);
  if (oracleSet.size !== citumSet.size || oracleSet.size !== oracleOrderIds.length) {
    return null;
  }
  for (const id of oracleSet) {
    if (!citumSet.has(id)) return null;
  }

  if (arraysEqual(oracleOrderIds, citumOrderIds)) {
    return null;
  }

  const anonymousIds = oracleOrderIds.filter((id) => !hasPrimaryNames(testItems[id]));
  if (anonymousIds.length === 0) {
    return null;
  }

  const anonymousSet = new Set(anonymousIds);
  const oracleNamed = oracleOrderIds.filter((id) => !anonymousSet.has(id));
  const citumNamed = citumOrderIds.filter((id) => !anonymousSet.has(id));
  const oracleAnonymous = oracleOrderIds.filter((id) => anonymousSet.has(id));
  const citumAnonymous = citumOrderIds.filter((id) => anonymousSet.has(id));

  if (!arraysEqual(oracleNamed, citumNamed) || !arraysEqual(oracleAnonymous, citumAnonymous)) {
    return null;
  }

  return {
    divergenceId: DIV_004_ID,
    scopes: divergenceRule.scopes || [],
    tags: divergenceRule.tags || [],
    note: divergenceRule.note || null,
    oracleOrderIds,
    citumOrderIds,
    anonymousIds,
  };
}

function maskNumericCitationLabels(text, labels) {
  let masked = normalizeText(text || '');
  const sorted = [...new Set(labels.filter((label) => Number.isInteger(label) && label > 0))]
    .sort((left, right) => String(right).length - String(left).length);

  for (const label of sorted) {
    const pattern = new RegExp(`(^|[\\[(;,\\-]\\s*)${label}(?=$|[\\])\\s;,\\-])`, 'g');
    masked = masked.replace(pattern, '$1#');
  }

  return normalizeText(masked);
}

function explainCitationMismatchFromDiv004(citationEntry, citationFixture, divergenceInfo) {
  if (!citationEntry || citationEntry.match || !citationFixture || !divergenceInfo) {
    return null;
  }

  const oracleLabelMap = buildNumericLabelMap(divergenceInfo.oracleOrderIds);
  const citumLabelMap = buildNumericLabelMap(divergenceInfo.citumOrderIds);
  const itemIds = (citationFixture.items || [])
    .map((item) => item.id)
    .filter((id) => oracleLabelMap.has(id) && citumLabelMap.has(id));

  if (itemIds.length === 0) {
    return null;
  }

  const oracleLabels = itemIds.map((id) => oracleLabelMap.get(id));
  const citumLabels = itemIds.map((id) => citumLabelMap.get(id));
  const maskedOracle = maskNumericCitationLabels(citationEntry.oracle, oracleLabels);
  const maskedCitum = maskNumericCitationLabels(citationEntry.citum, citumLabels);

  if (maskedOracle !== maskedCitum) {
    return null;
  }

  return {
    divergenceId: DIV_004_ID,
    tag: 'sort-derived-numeric-citation-label',
    itemIds,
    oracleLabels,
    citumLabels,
  };
}

function getStructuredArchiveInfo(ref) {
  return ref?.['archive-info'] || ref?.archive_info || null;
}

function getArchiveFragments(ref) {
  const archiveInfo = getStructuredArchiveInfo(ref);
  if (!archiveInfo || typeof archiveInfo !== 'object') {
    return [];
  }

  return [
    archiveInfo.collection,
    archiveInfo.location,
    archiveInfo.name,
    archiveInfo.place,
  ].filter((value) => typeof value === 'string' && value.trim().length > 0);
}

function stripTrailingArchiveFragments(text, fragments) {
  let stripped = text || '';
  for (const fragment of [...fragments].reverse()) {
    const suffixPattern = new RegExp(`,\\s*${escapeRegex(fragment)}\\s*$`);
    stripped = stripped.replace(suffixPattern, '');
  }
  return stripped;
}

function normalizeAncientYear(text, ref, oracleText) {
  const year = ref?.issued?.['date-parts']?.[0]?.[0];
  if (!Number.isInteger(year) || year >= 0) {
    return text;
  }

  const bcYear = `${Math.abs(year)} BC`;
  if (!normalizeText(oracleText || '').includes(normalizeText(bcYear))) {
    return text;
  }

  return String(text || '').replace(String(year), bcYear);
}

function explainCitationMismatchFromDiv005(citationEntry, citationFixture, testItems, divergenceRule) {
  if (!citationEntry || citationEntry.match || !citationFixture || !divergenceRule) {
    return null;
  }

  const itemIds = (citationFixture.items || []).map((item) => item.id).filter(Boolean);
  if (itemIds.length !== 1) {
    return null;
  }

  const ref = testItems[itemIds[0]];
  if (!ref || ref.type !== 'manuscript') {
    return null;
  }

  const archiveFragments = getArchiveFragments(ref);
  if (archiveFragments.length === 0) {
    return null;
  }

  const strippedCitum = stripTrailingArchiveFragments(citationEntry.citum, archiveFragments);
  const normalizedCitum = normalizeAncientYear(strippedCitum, ref, citationEntry.oracle);
  const comparison = compareText(citationEntry.oracle, normalizedCitum);
  if (!comparison.match || comparison.caseMismatch) {
    return null;
  }

  return {
    divergenceId: DIV_005_ID,
    tag: 'structured-archival-manuscript-detail',
    itemIds,
    archiveFragments,
    yearNormalized: strippedCitum !== normalizedCitum,
  };
}

function buildAdjustedOracleResult(rawResults, testCitations, testItems, divergenceInfo, div005Rule) {
  const adjustedCitationEntries = (rawResults.citations?.entries || []).map((entry, index) => {
    const div004Adjustment = explainCitationMismatchFromDiv004(
      entry,
      testCitations[index],
      divergenceInfo
    );
    const div005Adjustment = explainCitationMismatchFromDiv005(
      entry,
      testCitations[index],
      testItems,
      div005Rule
    );
    const appliedDivergence = div004Adjustment || div005Adjustment;
    return {
      ...entry,
      rawMatch: entry.match,
      match: entry.match || Boolean(appliedDivergence),
      appliedDivergence,
    };
  });

  const adjustedCitationPassed = adjustedCitationEntries.filter((entry) => entry.match).length;
  const adjustedCitationTotal = rawResults.citations?.total || adjustedCitationEntries.length;
  const adjustedBibliographyEntries = (rawResults.bibliography?.entries || []).map((entry) => ({
    ...entry,
    rawMatch: entry.match,
    match: entry.match,
  }));
  const divergenceSummary = {};

  if (divergenceInfo) {
    const adjustedCitationCount = adjustedCitationEntries
      .filter((entry) => entry.appliedDivergence?.divergenceId === DIV_004_ID)
      .length;
    divergenceSummary[DIV_004_ID] = {
      scopes: divergenceInfo.scopes,
      tags: divergenceInfo.tags,
      note: divergenceInfo.note,
      adjustedCitations: adjustedCitationCount,
      bibliographyOrderDifference: true,
      anonymousIds: divergenceInfo.anonymousIds,
    };
  }

  const div005Adjustments = adjustedCitationEntries
    .map((entry) => entry.appliedDivergence)
    .filter((entry) => entry?.divergenceId === DIV_005_ID);
  if (div005Rule && div005Adjustments.length > 0) {
    divergenceSummary[DIV_005_ID] = {
      scopes: div005Rule.scopes || [],
      tags: div005Rule.tags || [],
      note: div005Rule.note || null,
      adjustedCitations: div005Adjustments.length,
      itemIds: [...new Set(div005Adjustments.flatMap((entry) => entry.itemIds || []))],
    };
  }

  return {
    citations: {
      ...(rawResults.citations || {}),
      passed: adjustedCitationPassed,
      failed: Math.max(0, adjustedCitationTotal - adjustedCitationPassed),
      entries: adjustedCitationEntries,
    },
    bibliography: {
      ...(rawResults.bibliography || {}),
      entries: adjustedBibliographyEntries,
    },
    divergenceSummary,
  };
}

function attachRegisteredDivergenceAdjustments(rawResults, oracleBibliography, citumOrderIds, testItems, testCitations) {
  const hasCitationFailures = (rawResults?.citations?.failed || 0) > 0;
  const hasBibliographyFailures = (rawResults?.bibliography?.failed || 0) > 0;
  const shouldInspectOrderDifference = (
    hasCitationFailures || hasBibliographyFailures
  ) && Array.isArray(citumOrderIds) && citumOrderIds.length > 0;

  const policy = loadVerificationPolicy();
  const div005Rule = resolveRegisteredDivergence(policy, DIV_005_ID);

  if (!shouldInspectOrderDifference) {
    return {
      ...rawResults,
      bibliographyOrder: null,
      adjusted: buildAdjustedOracleResult(rawResults, testCitations, testItems, null, div005Rule),
    };
  }

  const divergenceRule = resolveRegisteredDivergence(policy, DIV_004_ID);
  const divergenceInfo = detectDiv004OrderDifference(
    oracleBibliography,
    citumOrderIds,
    testItems,
    divergenceRule
  );

  return {
    ...rawResults,
    bibliographyOrder: divergenceInfo
      ? {
          oracleOrderIds: divergenceInfo.oracleOrderIds,
          citumOrderIds: divergenceInfo.citumOrderIds,
          appliedDivergence: divergenceInfo.divergenceId,
        }
      : null,
    adjusted: buildAdjustedOracleResult(rawResults, testCitations, testItems, divergenceInfo, div005Rule),
  };
}

module.exports = {
  DIV_004_ID,
  DIV_005_ID,
  attachRegisteredDivergenceAdjustments,
  buildAdjustedOracleResult,
  buildNumericLabelMap,
  buildReferenceOrderIds,
  detectDiv004OrderDifference,
  explainCitationMismatchFromDiv004,
  explainCitationMismatchFromDiv005,
};
