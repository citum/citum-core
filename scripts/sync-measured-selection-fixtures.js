#!/usr/bin/env node
/*
 * Synchronize output-driven measured-selection fixtures from curated CSL JSON.
 *
 * Source items are allowlisted in tests/fixtures/measured-selection-imports.json.
 * The script copies those CSL JSON items into references-expanded.json and
 * references-heldout.json with stable TLIB-* IDs, removing previously generated
 * TLIB-* entries before writing. Optional type overrides materialize Zotero
 * Extra type hints as top-level CSL item types so citeproc-js and Citum see
 * the same effective type at the migration boundary.
 */

'use strict';

const fs = require('fs');
const path = require('path');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const IMPORTS_PATH = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'measured-selection-imports.json');
const SELECTION_PATH = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json');
const HELDOUT_PATH = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-heldout.json');

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function sourceItemsById(sourceDir, sourceFile) {
  const sourcePath = path.join(PROJECT_ROOT, sourceDir, sourceFile);
  const data = readJson(sourcePath);
  const items = Array.isArray(data)
    ? data
    : Array.isArray(data.items)
      ? data.items
      : Object.values(data).filter((value) => value && typeof value === 'object');
  return new Map(items.map((item) => [item.id, item]));
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function importedItem(importSpec, sourceDir) {
  const sourceMap = sourceItemsById(sourceDir, importSpec.source);
  const source = sourceMap.get(importSpec.source_id);
  if (!source) {
    throw new Error(`source item not found: ${importSpec.source} ${importSpec.source_id}`);
  }
  const item = clone(source);
  item.id = importSpec.id;
  if (importSpec.type) {
    item.type = importSpec.type;
  }
  return item;
}

function stripGeneratedEntries(fixture) {
  return Object.fromEntries(
    Object.entries(fixture).filter(([key]) => key === 'comment' || !key.startsWith('TLIB-'))
  );
}

function applyImports(fixturePath, importSpecs, sourceDir) {
  const fixture = stripGeneratedEntries(readJson(fixturePath));
  for (const importSpec of importSpecs) {
    if (!importSpec.id.startsWith('TLIB-')) {
      throw new Error(`imported fixture id must use TLIB-* prefix: ${importSpec.id}`);
    }
    fixture[importSpec.id] = importedItem(importSpec, sourceDir);
  }
  return fixture;
}

function normalizedTitle(item) {
  return String(item.title || '').trim().toLowerCase();
}

function assertDisjoint(selection, heldout) {
  const selectionIds = new Set(Object.keys(selection));
  const selectionTitles = new Set(
    Object.values(selection).map(normalizedTitle).filter((title) => title.length > 0)
  );
  for (const [id, item] of Object.entries(heldout)) {
    if (id === 'comment') continue;
    if (selectionIds.has(id)) {
      throw new Error(`held-out item reuses selection id: ${id}`);
    }
    const title = normalizedTitle(item);
    if (title && selectionTitles.has(title)) {
      throw new Error(`held-out item ${id} reuses selection title: ${item.title}`);
    }
  }
}

function assertUniqueImports(imports) {
  const seenIds = new Set();
  const seenSources = new Set();
  for (const section of ['selection', 'heldout']) {
    for (const spec of imports[section] || []) {
      if (seenIds.has(spec.id)) {
        throw new Error(`duplicate imported fixture id: ${spec.id}`);
      }
      seenIds.add(spec.id);
      const sourceKey = `${spec.source}:${spec.source_id}`;
      if (seenSources.has(sourceKey)) {
        throw new Error(`source item imported twice: ${sourceKey}`);
      }
      seenSources.add(sourceKey);
    }
  }
}

function main() {
  const imports = readJson(IMPORTS_PATH);
  if (imports.version !== 1) {
    throw new Error('measured-selection-imports.json version must be 1');
  }
  assertUniqueImports(imports);
  const selection = applyImports(SELECTION_PATH, imports.selection || [], imports.source_dir);
  const heldout = applyImports(HELDOUT_PATH, imports.heldout || [], imports.source_dir);
  assertDisjoint(selection, heldout);
  writeJson(SELECTION_PATH, selection);
  writeJson(HELDOUT_PATH, heldout);
  console.log(
    `Synchronized ${(imports.selection || []).length} selection and ${(imports.heldout || []).length} held-out fixture items.`
  );
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(`sync-measured-selection-fixtures failed: ${error.message}`);
    process.exit(1);
  }
}
