#!/usr/bin/env node
/**
 * Migrate Citum locale v1 to v2 format
 *
 * Converts v1 term structure (singular/plural fields, long/short strings)
 * to v2 ICU message format with kebab-case keys.
 *
 * Usage:
 *   node scripts/migrate-locale-v1-to-v2.js locales/en-US.yaml > /tmp/en-US-v2.yaml
 *
 * Exit codes:
 *   0 - Success (output to stdout)
 *   1 - Fatal error (file not found, parse error)
 */

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');

if (process.argv.length < 3) {
  console.error('Usage: node migrate-locale-v1-to-v2.js <locale-file>');
  process.exit(1);
}

const localeFile = process.argv[2];

// Read and parse the locale YAML
let localeData;
try {
  const content = fs.readFileSync(localeFile, 'utf8');
  localeData = yaml.load(content);
} catch (err) {
  console.error(`Error reading or parsing ${localeFile}:`, err.message);
  process.exit(1);
}

// Initialize v2 structures
const messages = {};
const legacyTermAliases = {};
const dateFormats = {};
const grammarOptions = {};

// Process terms to generate messages and aliases
if (localeData.terms) {
  for (const [termKey, termValue] of Object.entries(localeData.terms)) {
    if (!termValue) continue;

    // Detect structure type
    if (typeof termValue === 'object') {
      // Handle singular/plural structure
      if (termValue.singular !== undefined && termValue.plural !== undefined) {
        const singular = termValue.singular;
        const plural = termValue.plural;
        const messageKey = `term.${termKey.replace(/_/g, '-')}`;
        messages[messageKey] = `{count, plural, one {${singular}} other {${plural}}}`;
        legacyTermAliases[termKey] = messageKey;
      }
      // Handle long/short structure (bare long string as fallback)
      else if (termValue.long !== undefined) {
        const longStr = termValue.long;
        const messageKey = `term.${termKey.replace(/_/g, '-')}`;
        messages[messageKey] = longStr;
        legacyTermAliases[termKey] = messageKey;
      }
    } else if (typeof termValue === 'string') {
      // Bare string value
      const messageKey = `term.${termKey.replace(/_/g, '-')}`;
      messages[messageKey] = termValue;
      legacyTermAliases[termKey] = messageKey;
    }
  }
}

// Add schema version
localeData['locale-schema-version'] = '2';

// Add v2 blocks if not present
if (!localeData.messages) {
  localeData.messages = messages;
}
if (!localeData['legacy-term-aliases']) {
  localeData['legacy-term-aliases'] = legacyTermAliases;
}

// Output as YAML
try {
  const output = yaml.dump(localeData, {
    lineWidth: 120,
    noRefs: true,
    sortKeys: false,
  });
  process.stdout.write(output);
} catch (err) {
  console.error('Error serializing to YAML:', err.message);
  process.exit(1);
}

process.exit(0);
