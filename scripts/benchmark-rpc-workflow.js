/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

/**
 * @file benchmark-rpc-workflow.js
 * @description Simulates a word processor (LibreOffice, Zotero, etc.) workflow using the Citum RPC server.
 * Generates a large bibliography, inserts citations sequentially, and periodically updates
 * the bibliography, measuring performance metrics (latency p50/p95/p99).
 */

const { spawn } = require('node:child_process');
const readline = require('node:readline');
const { performance } = require('node:perf_hooks');
const path = require('node:path');
const fs = require('node:fs');

// --- Configuration ---
const REFS_COUNT = 500;
const CITATIONS_COUNT = 100;
const REFRESH_INTERVAL = 20;
const STYLE_PATH = 'styles/embedded/apa-7th.yaml';
const SERVER_BIN = path.resolve(__dirname, '../target/release/citum-server');

// --- State ---
let requestId = 0;
const pendingRequests = new Map();

async function runBenchmark() {
  console.log('====================================================');
  console.log('   Citum RPC: Word Processor Workflow Benchmark');
  console.log('====================================================');

  if (!fs.existsSync(SERVER_BIN)) {
    console.error(`Error: Server binary not found at ${SERVER_BIN}`);
    console.error('Please run: cargo build --release -p citum-server');
    process.exit(1);
  }

  console.log(`[Config] Style:      ${STYLE_PATH}`);
  console.log(`[Config] References: ${REFS_COUNT}`);
  console.log(`[Config] Citations:  ${CITATIONS_COUNT}`);
  console.log(`[Config] Refresh:    Every ${REFRESH_INTERVAL} citations`);
  console.log('----------------------------------------------------');

  const server = spawn(SERVER_BIN, [], { stdio: ['pipe', 'pipe', 'inherit'] });
  const rl = readline.createInterface({ input: server.stdout });

  rl.on('line', (line) => {
    try {
      const response = JSON.parse(line);
      const resolver = pendingRequests.get(response.id);
      if (resolver) {
        pendingRequests.delete(response.id);
        resolver(response);
      }
    } catch (e) {
      console.error('Failed to parse server response:', line);
    }
  });

  const rpc = async (method, params) => {
    const id = ++requestId;
    const start = performance.now();
    return new Promise((resolve) => {
      pendingRequests.set(id, (res) => {
        const end = performance.now();
        resolve({ ...res, duration: end - start });
      });
      server.stdin.write(JSON.stringify({ id, method, params }) + '\n');
    });
  };

  // 1. Generate large dataset of references (Native Citum format)
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

  const citationTimes = [];
  const bibTimes = [];

  console.log('Simulating authoring workflow...');

  for (let i = 0; i < CITATIONS_COUNT; i++) {
    // Simulate inserting a citation
    const citation = {
      id: `cite-${i}`,
      items: [{ id: `item-${i % REFS_COUNT}` }]
    };

    const res = await rpc('render_citation', {
      style_path: STYLE_PATH,
      refs: refs,
      citation: citation
    });

    if (res.error) {
        console.error(`\nError rendering citation ${i}:`, res.error);
        break;
    }
    citationTimes.push(res.duration);

    // Periodically update the bibliography
    if ((i + 1) % REFRESH_INTERVAL === 0) {
      const bRes = await rpc('render_bibliography', {
        style_path: STYLE_PATH,
        refs: refs
      });
      if (bRes.error) {
          console.error(`\nError rendering bibliography at index ${i}:`, bRes.error);
          break;
      }
      bibTimes.push(bRes.duration);
      console.log(`[Progress] Citation ${String(i + 1).padStart(3)}/${CITATIONS_COUNT} | Bibliography refreshed (${bRes.duration.toFixed(2)}ms)`);
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
  console.log('             Performance Statistics');
  console.log('----------------------------------------------------');
  console.log(`render_citation:     ${stats(citationTimes)}`);
  console.log(`render_bibliography: ${stats(bibTimes)}`);
  console.log('----------------------------------------------------');

  server.kill();
  process.exit(0);
}

runBenchmark().catch(err => {
  console.error('\nFatal error:', err);
  process.exit(1);
});
