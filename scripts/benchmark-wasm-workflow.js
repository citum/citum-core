/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

/**
 * @file benchmark-wasm-workflow.js
 * @description Simulates a word processor workflow using Citum WASM bindings.
 * Measures performance metrics for citation and bibliography rendering.
 */

const { performance } = require('node:perf_hooks');
const fs = require('node:fs');
const path = require('node:path');

// --- Configuration ---
const args = process.argv.slice(2);
const getArg = (name, defaultValue) => {
  const index = args.findIndex(arg => arg === name);
  return (index !== -1 && args[index + 1]) ? args[index + 1] : defaultValue;
};

const CI_MODE = args.includes('--ci');
const REFS_COUNT = parseInt(getArg('--refs', CI_MODE ? '50' : '500'), 10);
const CITATIONS_COUNT = parseInt(getArg('--cites', CI_MODE ? '20' : '100'), 10);
const REFRESH_INTERVAL = parseInt(getArg('--interval', CI_MODE ? '5' : '20'), 10);
const STYLE_PATH = getArg('--style', 'styles/embedded/apa-7th.yaml');

function runBenchmark() {
  // Lazy-load WASM so a missing artifact gives a clear, actionable error.
  let renderCitation, renderBibliography, validateStyle;
  const wasmJs = path.resolve(__dirname, '../crates/citum-bindings/pkg/citum_bindings.js');
  const wasmBin = path.resolve(__dirname, '../crates/citum-bindings/pkg/citum_bindings_bg.wasm');
  if (!fs.existsSync(wasmJs) || !fs.existsSync(wasmBin)) {
    console.error(
      'WASM not built. Run: wasm-pack build --target nodejs --features full-wasm  (in crates/citum-bindings/)'
    );
    process.exit(1);
  }
  try {
    ({ renderCitation, renderBibliography, validateStyle } = require(wasmJs));
  } catch (e) {
    console.error('Failed to load WASM bindings:', e.message);
    process.exit(1);
  }

  console.log('====================================================');
  console.log('   Citum WASM: Word Processor Workflow Benchmark');
  console.log('====================================================');

  const styleYaml = fs.readFileSync(path.resolve(__dirname, '..', STYLE_PATH), 'utf8');

  console.log(`[Config] Style:      ${STYLE_PATH}`);
  console.log(`[Config] References: ${REFS_COUNT}`);
  console.log(`[Config] Citations:  ${CITATIONS_COUNT}`);
  console.log(`[Config] Refresh:    Every ${REFRESH_INTERVAL} citations`);
  if (CI_MODE) console.log('[Config] Mode:       CI (reduced params)');
  console.log('----------------------------------------------------');

  // 1. Warmup / Validation
  try {
    validateStyle(styleYaml);
    console.log('[Status] Style validated.');
  } catch (e) {
    console.error('Style validation failed:', e);
    process.exit(1);
  }

  // 2. Generate large dataset of references
  const refs = {};
  for (let i = 0; i < REFS_COUNT; i++) {
    refs[`item-${i}`] = {
      id: `item-${i}`,
      class: "monograph",
      type: "book",
      title: `The Great Book of Things Part ${i}`,
      author: [{ family: `Author${i}`, given: "Alice" }],
      issued: `${2000 + (i % 24)}`
    };
  }
  const refsJson = JSON.stringify(refs);

  const citationTimes = [];
  const bibTimes = [];

  console.log('Simulating authoring workflow...');

  for (let i = 0; i < CITATIONS_COUNT; i++) {
    // Simulate inserting a citation
    const citation = {
      id: `cite-${i}`,
      items: [{ id: `item-${i % REFS_COUNT}` }]
    };
    const citationJson = JSON.stringify(citation);

    const start = performance.now();
    try {
      renderCitation(styleYaml, refsJson, citationJson, null);
      const end = performance.now();
      citationTimes.push(end - start);
    } catch (e) {
      console.error(`\nError rendering citation ${i}:`, e);
      break;
    }

    // Periodically update the bibliography
    if ((i + 1) % REFRESH_INTERVAL === 0) {
      const bStart = performance.now();
      try {
        renderBibliography(styleYaml, refsJson);
        const bEnd = performance.now();
        bibTimes.push(bEnd - bStart);
        console.log(`[Progress] Citation ${String(i + 1).padStart(3)}/${CITATIONS_COUNT} | Bibliography refreshed (${(bEnd - bStart).toFixed(2)}ms)`);
      } catch (e) {
        console.error(`\nError rendering bibliography at index ${i}:`, e);
        break;
      }
    } else if ((i + 1) % 10 === 0 || i === 0) {
      console.log(`[Progress] Citation ${String(i + 1).padStart(3)}/${CITATIONS_COUNT}...`);
    }
  }

  console.log('\n\nSimulation complete.');

  const stats = (times) => {
    if (times.length === 0) return 'N/A';
    times.sort((a, b) => a - b);
    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    const p50 = times[Math.floor(times.length * 0.5)];
    const p95 = times[Math.floor(times.length * 0.95)];
    const p99 = times[Math.floor(times.length * 0.99)];
    return `Avg: ${avg.toFixed(2).padStart(6)}ms | P50: ${p50.toFixed(2).padStart(6)}ms | P95: ${p95.toFixed(2).padStart(6)}ms | P99: ${p99.toFixed(2).padStart(6)}ms`;
  };

  console.log('----------------------------------------------------');
  console.log('             Performance Statistics (WASM)');
  console.log('----------------------------------------------------');
  console.log(`render_citation:     ${stats(citationTimes)}`);
  console.log(`render_bibliography: ${stats(bibTimes)}`);
  console.log('----------------------------------------------------');
}

try { runBenchmark(); } catch (err) {
  console.error('\nFatal error:', err);
  process.exit(1);
}
