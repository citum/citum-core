#!/usr/bin/env node
/**
 * Node wrapper for the host-neutral template inference core.
 *
 * Reads style XML, locale XML, and fixture data from disk, then delegates the
 * actual inference work to `template-inferrer-core.js`.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const core = require('./template-inferrer-core');

/**
 * Load CSL locale XML by language code.
 * Falls back to en-US if the requested locale is not found.
 */
function loadLocale(lang = 'en-US') {
  const localePath = path.join(__dirname, '..', `locales-${lang}.xml`);
  if (fs.existsSync(localePath)) {
    return fs.readFileSync(localePath, 'utf8');
  }

  const fallback = path.join(__dirname, '..', 'locales-en-US.xml');
  if (fs.existsSync(fallback)) {
    return fs.readFileSync(fallback, 'utf8');
  }

  throw new Error(`Locale not found: ${lang}`);
}

/**
 * Load test fixture items for analysis.
 * Filter out the comment field and return a map of ID → reference data.
 */
function loadFixtures() {
  const fixturesPath = path.join(
    __dirname,
    '..',
    '..',
    'tests',
    'fixtures',
    'references-expanded.json',
  );
  if (!fs.existsSync(fixturesPath)) {
    throw new Error(`Fixtures not found at ${fixturesPath}`);
  }

  const fixturesData = JSON.parse(fs.readFileSync(fixturesPath, 'utf8'));
  return Object.fromEntries(
    Object.entries(fixturesData).filter(([key]) => key !== 'comment'),
  );
}

/**
 * Infer a template by reading the required inputs from disk first.
 */
function inferTemplate(stylePath, section = 'bibliography') {
  if (!fs.existsSync(stylePath)) return null;

  const styleXml = fs.readFileSync(stylePath, 'utf8');
  const testItems = loadFixtures();
  const localeXml = loadLocale('en-US');

  return core.inferTemplateFromInputs({
    styleXml,
    section,
    testItems,
    localeXml,
  });
}

module.exports = {
  ...core,
  inferTemplate,
  loadLocale,
  loadFixtures,
};
