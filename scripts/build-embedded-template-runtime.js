#!/usr/bin/env node
/**
 * Build the host-neutral JS bundle embedded by `citum-migrate`.
 *
 * The generated file is committed so Cargo builds do not depend on npm or a JS
 * toolchain. Regenerate it whenever the inference core or citeproc bundle
 * changes.
 */

'use strict';

const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.join(__dirname, '..');
const outputPath = path.join(
  repoRoot,
  'crates',
  'citum-migrate',
  'js',
  'embedded-template-runtime.js',
);

const moduleSources = {
  citeproc: fs.readFileSync(
    path.join(repoRoot, 'scripts', 'node_modules', 'citeproc', 'citeproc_commonjs.js'),
    'utf8',
  ),
  './component-parser': fs.readFileSync(
    path.join(repoRoot, 'scripts', 'lib', 'component-parser.js'),
    'utf8',
  ),
  './template-inferrer-core': fs.readFileSync(
    path.join(repoRoot, 'scripts', 'lib', 'template-inferrer-core.js'),
    'utf8',
  ),
};

const bundle = `/* eslint-disable */
/*
 * GENERATED FILE - DO NOT EDIT DIRECTLY.
 *
 * Regenerate with:
 *   node scripts/build-embedded-template-runtime.js
 */
(function () {
  'use strict';

  const MODULE_SOURCES = ${JSON.stringify(moduleSources, null, 2)};
  const MODULE_CACHE = Object.create(null);

  function __require(id) {
    if (Object.prototype.hasOwnProperty.call(MODULE_CACHE, id)) {
      return MODULE_CACHE[id].exports;
    }
    if (!Object.prototype.hasOwnProperty.call(MODULE_SOURCES, id)) {
      throw new Error('Unknown embedded module: ' + id);
    }

    const module = { exports: {} };
    MODULE_CACHE[id] = module;
    const factory = new Function('module', 'exports', 'require', MODULE_SOURCES[id]);
    factory(module, module.exports, __require);
    return module.exports;
  }

  const core = __require('./template-inferrer-core');

  globalThis.infer_template_fragment = function inferTemplateFragment(input) {
    const args = typeof input === 'string' ? JSON.parse(input) : input;
    const fragment = core.inferTemplateFragment(args);
    return fragment === null ? null : JSON.stringify(fragment);
  };
}());
`;

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, bundle);
console.log(outputPath);
