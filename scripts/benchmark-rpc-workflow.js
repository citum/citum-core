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

// --- Configuration (with CLI overrides) ---
const args = process.argv.slice(2);
const getArg = (name, defaultValue) => {
  const index = args.findIndex(arg => arg === name);
  return (index !== -1 && args[index + 1]) ? args[index + 1] : defaultValue;
};

const REFS_COUNT = parseInt(getArg('--refs', '500'), 10);
const CITATIONS_COUNT = parseInt(getArg('--cites', '100'), 10);
const REFRESH_INTERVAL = parseInt(getArg('--interval', '20'), 10);
const STYLE_PATH = getArg('--style', 'styles/embedded/apa-7th.yaml');
const SERVER_NAME = 'citum-server';
const SERVER_BIN = path.resolve(__dirname, '../target/release/citum-server');

function isExecutable(filePath) {
  try {
    fs.accessSync(filePath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function resolveServerBinary() {
  const pathEntries = (process.env.PATH || '').split(path.delimiter).filter(Boolean);

  for (const entry of pathEntries) {
    const candidate = path.join(entry, SERVER_NAME);
    if (isExecutable(candidate)) {
      return candidate;
    }

    if (process.platform === 'win32') {
      const pathext = (process.env.PATHEXT || '.EXE;.CMD;.BAT;.COM')
        .split(path.delimiter)
        .filter(Boolean);
      for (const ext of pathext) {
        const extCandidate = `${candidate}${ext}`;
        if (isExecutable(extCandidate)) {
          return extCandidate;
        }
      }
    }
  }

  if (isExecutable(SERVER_BIN)) {
    return SERVER_BIN;
  }

  return null;
}

// --- State ---
let requestId = 0;
const pendingRequests = new Map();

function resolvePendingRequests(response) {
  for (const [id, resolver] of pendingRequests.entries()) {
    pendingRequests.delete(id);
    resolver(response);
  }
}

async function runBenchmark() {
  console.log('====================================================');
  console.log('   Citum RPC: Word Processor Workflow Benchmark');
  console.log('====================================================');

  const serverBin = resolveServerBinary();
  if (!serverBin) {
    console.error(`Error: Server binary not found on PATH or at ${SERVER_BIN}`);
    console.error('Please run: cargo build --release -p citum-server or install citum-server on PATH');
    process.exit(1);
  }

  console.log(`[Config] Style:      ${STYLE_PATH}`);
  console.log(`[Config] References: ${REFS_COUNT}`);
  console.log(`[Config] Citations:  ${CITATIONS_COUNT}`);
  console.log(`[Config] Refresh:    Every ${REFRESH_INTERVAL} citations`);
  console.log('----------------------------------------------------');

  const server = spawn(serverBin, [], { stdio: ['pipe', 'pipe', 'inherit'] });
  const rl = readline.createInterface({ input: server.stdout });
  let serverClosed = false;
  let serverFailure = null;

  server.on('error', (error) => {
    serverClosed = true;
    serverFailure = `Failed to start server: ${error.message}`;
    resolvePendingRequests({ error: serverFailure });
  });

  server.on('exit', (code, signal) => {
    serverClosed = true;
    serverFailure = signal
      ? `Server exited unexpectedly (signal ${signal})`
      : `Server exited unexpectedly (exit code ${code ?? 'unknown'})`;
  });

  rl.on('close', () => {
    if (pendingRequests.size > 0) {
      resolvePendingRequests({ error: serverFailure || 'Server output closed unexpectedly' });
    }
  });

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

      if (serverClosed || server.stdin.destroyed) {
        pendingRequests.delete(id);
        resolve({ error: 'Server is not available', duration: 0 });
        return;
      }

      server.stdin.write(JSON.stringify({ id, method, params }) + '\n', (error) => {
        if (!error) {
          return;
        }

        if (pendingRequests.has(id)) {
          pendingRequests.delete(id);
          resolve({ error: `Failed to send request: ${error.message}`, duration: 0 });
        }
      });
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
