import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { runJsrPackageSmoke } from './jsr-package-smoke-cases.mjs';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const args = globalThis.Deno ? Deno.args : process.argv.slice(2);
const packageDir = path.resolve(args[0] ?? path.join(repoRoot, 'target/jsr/citum'));
const moduleUrl = pathToFileURL(path.join(packageDir, 'citum_bindings.js')).href;
const wasmBytes = await readFile(path.join(packageDir, 'citum_bindings_bg.wasm'));
const api = await import(moduleUrl);

await api.default({ module_or_path: wasmBytes });
const result = runJsrPackageSmoke(api);
console.log(`JSR package smoke passed: ${JSON.stringify(result)}`);
