'use strict';

const {
  findRefMatchForEntry,
  hasPrimaryNames,
  normalizeText,
} = require('../oracle-utils');
const {
  loadVerificationPolicy,
  resolveRegisteredDivergence,
} = require('./verification-policy');

const DIV_004_ID = 'div-004';

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

function detectDiv004OrderDifference(oracleBibliography, cslnOrderIds, testItems, divergenceRule) {
  if (!divergenceRule || !Array.isArray(oracleBibliography) || !Array.isArray(cslnOrderIds)) {
    return null;
  }

  const oracleOrderIds = buildReferenceOrderIds(oracleBibliography, testItems);
  if (oracleOrderIds.length === 0 || oracleOrderIds.length !== cslnOrderIds.length) {
    return null;
  }

  const oracleSet = new Set(oracleOrderIds);
  const cslnSet = new Set(cslnOrderIds);
  if (oracleSet.size !== cslnSet.size || oracleSet.size !== oracleOrderIds.length) {
    return null;
  }
  for (const id of oracleSet) {
    if (!cslnSet.has(id)) return null;
  }

  if (arraysEqual(oracleOrderIds, cslnOrderIds)) {
    return null;
  }

  const anonymousIds = oracleOrderIds.filter((id) => !hasPrimaryNames(testItems[id]));
  if (anonymousIds.length === 0) {
    return null;
  }

  const anonymousSet = new Set(anonymousIds);
  const oracleNamed = oracleOrderIds.filter((id) => !anonymousSet.has(id));
  const cslnNamed = cslnOrderIds.filter((id) => !anonymousSet.has(id));
  const oracleAnonymous = oracleOrderIds.filter((id) => anonymousSet.has(id));
  const cslnAnonymous = cslnOrderIds.filter((id) => anonymousSet.has(id));

  if (!arraysEqual(oracleNamed, cslnNamed) || !arraysEqual(oracleAnonymous, cslnAnonymous)) {
    return null;
  }

  return {
    divergenceId: DIV_004_ID,
    scopes: divergenceRule.scopes || [],
    tags: divergenceRule.tags || [],
    note: divergenceRule.note || null,
    oracleOrderIds,
    cslnOrderIds,
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
  const cslnLabelMap = buildNumericLabelMap(divergenceInfo.cslnOrderIds);
  const itemIds = (citationFixture.items || [])
    .map((item) => item.id)
    .filter((id) => oracleLabelMap.has(id) && cslnLabelMap.has(id));

  if (itemIds.length === 0) {
    return null;
  }

  const oracleLabels = itemIds.map((id) => oracleLabelMap.get(id));
  const cslnLabels = itemIds.map((id) => cslnLabelMap.get(id));
  const maskedOracle = maskNumericCitationLabels(citationEntry.oracle, oracleLabels);
  const maskedCsln = maskNumericCitationLabels(citationEntry.csln, cslnLabels);

  if (maskedOracle !== maskedCsln) {
    return null;
  }

  return {
    divergenceId: DIV_004_ID,
    tag: 'sort-derived-numeric-citation-label',
    itemIds,
    oracleLabels,
    cslnLabels,
  };
}

function buildAdjustedOracleResult(rawResults, testCitations, divergenceInfo) {
  const adjustedCitationEntries = (rawResults.citations?.entries || []).map((entry, index) => {
    const appliedDivergence = explainCitationMismatchFromDiv004(
      entry,
      testCitations[index],
      divergenceInfo
    );
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
    const adjustedCitationCount = adjustedCitationEntries.filter((entry) => entry.appliedDivergence).length;
    divergenceSummary[DIV_004_ID] = {
      scopes: divergenceInfo.scopes,
      tags: divergenceInfo.tags,
      note: divergenceInfo.note,
      adjustedCitations: adjustedCitationCount,
      bibliographyOrderDifference: true,
      anonymousIds: divergenceInfo.anonymousIds,
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

function attachRegisteredDivergenceAdjustments(rawResults, oracleBibliography, cslnOrderIds, testItems, testCitations) {
  const hasCitationFailures = (rawResults?.citations?.failed || 0) > 0;
  const hasBibliographyFailures = (rawResults?.bibliography?.failed || 0) > 0;
  const shouldInspectOrderDifference = (
    hasCitationFailures || hasBibliographyFailures
  ) && Array.isArray(cslnOrderIds) && cslnOrderIds.length > 0;

  if (!shouldInspectOrderDifference) {
    return {
      ...rawResults,
      bibliographyOrder: null,
      adjusted: buildAdjustedOracleResult(rawResults, testCitations, null),
    };
  }

  const policy = loadVerificationPolicy();
  const divergenceRule = resolveRegisteredDivergence(policy, DIV_004_ID);
  const divergenceInfo = detectDiv004OrderDifference(
    oracleBibliography,
    cslnOrderIds,
    testItems,
    divergenceRule
  );

  return {
    ...rawResults,
    bibliographyOrder: divergenceInfo
      ? {
          oracleOrderIds: divergenceInfo.oracleOrderIds,
          cslnOrderIds: divergenceInfo.cslnOrderIds,
          appliedDivergence: divergenceInfo.divergenceId,
        }
      : null,
    adjusted: buildAdjustedOracleResult(rawResults, testCitations, divergenceInfo),
  };
}

module.exports = {
  DIV_004_ID,
  attachRegisteredDivergenceAdjustments,
  buildAdjustedOracleResult,
  buildNumericLabelMap,
  buildReferenceOrderIds,
  detectDiv004OrderDifference,
  explainCitationMismatchFromDiv004,
  maskNumericCitationLabels,
};
