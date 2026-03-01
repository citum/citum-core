#!/usr/bin/env node

/*
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");

const DEFAULT_ISSUE_URL =
  "https://api.github.com/repos/typst/hayagriva/issues/327";
const DEFAULT_WAVE1_FILE = "tests/fixtures/csl-native-intake-wave1.json";

function parseArgs(argv) {
  const args = {
    issueUrl: DEFAULT_ISSUE_URL,
    issueFile: null,
    json: false,
    progressOnly: false,
    wave1File: DEFAULT_WAVE1_FILE,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--json") {
      args.json = true;
    } else if (arg === "--progress-only") {
      args.progressOnly = true;
    } else if (arg === "--issue-file") {
      args.issueFile = argv[i + 1];
      i += 1;
    } else if (arg === "--issue-url") {
      args.issueUrl = argv[i + 1];
      i += 1;
    } else if (arg === "--wave1-file") {
      args.wave1File = argv[i + 1];
      i += 1;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

function fetchJson(url) {
  return new Promise((resolve, reject) => {
    const request = https.get(
      url,
      {
        headers: {
          "User-Agent": "citum-csl-intake-report",
          Accept: "application/vnd.github+json",
        },
      },
      (response) => {
        if (response.statusCode !== 200) {
          reject(
            new Error(`Request failed with status ${response.statusCode}: ${url}`),
          );
          response.resume();
          return;
        }

        let body = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => {
          try {
            resolve(JSON.parse(body));
          } catch (error) {
            reject(error);
          }
        });
      },
    );

    request.on("error", reject);
  });
}

function parseIssueChecklist(body) {
  const entries = [];
  let section = "Uncategorized";
  let lastEntry = null;

  for (const rawLine of body.split("\n")) {
    const line = rawLine.trimEnd();

    if (line.startsWith("## ")) {
      section = line.slice(3).trim();
      lastEntry = null;
      continue;
    }

    const itemMatch = line.match(/^- \[([ xX])\] ([A-Za-z0-9_]+)/);
    if (itemMatch) {
      lastEntry = {
        section,
        id: itemMatch[2],
        upstream_checked: itemMatch[1].toLowerCase() === "x",
        notes: [],
      };
      entries.push(lastEntry);
      continue;
    }

    const noteMatch = line.match(/^  - (.+)$/);
    if (noteMatch && lastEntry) {
      lastEntry.notes.push(noteMatch[1]);
    }
  }

  return entries;
}

function classifyUpstreamState(entry) {
  const notes = entry.notes.join(" ").toLowerCase();
  if (notes.includes("diverging behavior")) return "diverging";
  if (notes.includes("we don't support")) return "unsupported";
  if (entry.upstream_checked || notes.includes("passes in #")) return "passes-upstream";
  return "open-upstream";
}

function findFixturePaths(root, id) {
  const humans = path.join(
    root,
    "tests",
    "csl-test-suite",
    "processor-tests",
    "humans",
    `${id}.txt`,
  );
  const machines = path.join(
    root,
    "tests",
    "csl-test-suite",
    "processor-tests",
    "machines",
    `${id}.json`,
  );

  return {
    human: fs.existsSync(humans)
      ? path.relative(root, humans)
      : null,
    machine: fs.existsSync(machines)
      ? path.relative(root, machines)
      : null,
  };
}

function countBy(items, keyFn) {
  const counts = {};
  for (const item of items) {
    const key = keyFn(item);
    counts[key] = (counts[key] || 0) + 1;
  }
  return counts;
}

function loadWave1Manifest(root, manifestPath) {
  const resolvedPath = path.resolve(root, manifestPath);
  if (!fs.existsSync(resolvedPath)) {
    return null;
  }

  return JSON.parse(fs.readFileSync(resolvedPath, "utf8"));
}

function buildWave1Progress(manifest) {
  const entries = manifest.entries || [];
  const integrateNow = entries.filter((entry) => entry.decision === "integrate-now");
  const complete = integrateNow.filter(
    (entry) => entry.current_coverage === "native-regression",
  );
  const partial = integrateNow.filter((entry) =>
    ["adjacent-native", "partial-native"].includes(entry.current_coverage),
  );
  const remaining = integrateNow.filter((entry) =>
    !["native-regression", "adjacent-native", "partial-native"].includes(
      entry.current_coverage,
    ),
  );

  return {
    metadata: manifest.metadata || {},
    summary: {
      total: entries.length,
      by_decision: countBy(entries, (entry) => entry.decision),
      by_current_coverage: countBy(entries, (entry) => entry.current_coverage),
      integrate_now: {
        total: integrateNow.length,
        complete: complete.length,
        partial: partial.length,
        remaining: remaining.length,
      },
    },
    lists: {
      complete: complete.map((entry) => entry.id),
      partial: partial.map((entry) => entry.id),
      remaining: remaining.map((entry) => entry.id),
      adapt_later: entries
        .filter((entry) => entry.decision === "adapt-later")
        .map((entry) => entry.id),
      exclude_for_now: entries
        .filter((entry) => entry.decision === "exclude-for-now")
        .map((entry) => entry.id),
    },
  };
}

function printIssueSummary(report) {
  console.log(`Issue: ${report.issue.title} (#${report.issue.number})`);
  console.log(`URL: ${report.issue.html_url}`);
  console.log(`Updated: ${report.issue.updated_at}`);
  console.log("");
  console.log(`Total checklist fixtures: ${report.summary.total}`);
  console.log(`With local human fixture: ${report.summary.with_human_fixture}`);
  console.log(`With local machine fixture: ${report.summary.with_machine_fixture}`);
  console.log("");
  console.log("By section:");
  for (const [section, count] of Object.entries(report.summary.by_section)) {
    console.log(`  ${section}: ${count}`);
  }
  console.log("");
  console.log("By upstream state:");
  for (const [state, count] of Object.entries(report.summary.by_upstream_state)) {
    console.log(`  ${state}: ${count}`);
  }
}

function printWave1Summary(wave1) {
  console.log("Wave 1 progress:");
  console.log(`  Total entries: ${wave1.summary.total}`);
  console.log(
    `  Integrate-now: ${wave1.summary.integrate_now.complete}/${wave1.summary.integrate_now.total} complete, ${wave1.summary.integrate_now.partial} partial, ${wave1.summary.integrate_now.remaining} remaining`,
  );
  console.log("");
  console.log("By decision:");
  for (const [decision, count] of Object.entries(wave1.summary.by_decision)) {
    console.log(`  ${decision}: ${count}`);
  }
  console.log("");
  console.log("By current coverage:");
  for (const [coverage, count] of Object.entries(wave1.summary.by_current_coverage)) {
    console.log(`  ${coverage}: ${count}`);
  }

  if (wave1.lists.partial.length > 0) {
    console.log("");
    console.log("Partial integrate-now cases:");
    for (const id of wave1.lists.partial) {
      console.log(`  ${id}`);
    }
  }

  if (wave1.lists.remaining.length > 0) {
    console.log("");
    console.log("Remaining integrate-now cases:");
    for (const id of wave1.lists.remaining) {
      console.log(`  ${id}`);
    }
  }
}

function printTextReport(report) {
  if (report.issue && report.summary) {
    printIssueSummary(report);
  }

  if (report.wave1) {
    if (report.issue && report.summary) {
      console.log("");
    }
    printWave1Summary(report.wave1);
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const root = path.resolve(__dirname, "..");
  const wave1Manifest = loadWave1Manifest(root, args.wave1File);

  const report = {
    generated_at: new Date().toISOString(),
    wave1: wave1Manifest ? buildWave1Progress(wave1Manifest) : null,
  };

  if (!args.progressOnly) {
    const issue = args.issueFile
      ? JSON.parse(fs.readFileSync(path.resolve(root, args.issueFile), "utf8"))
      : await fetchJson(args.issueUrl);

    const entries = parseIssueChecklist(issue.body).map((entry) => {
      const fixturePaths = findFixturePaths(root, entry.id);
      return {
        ...entry,
        upstream_state: classifyUpstreamState(entry),
        fixture_paths: fixturePaths,
        present_locally: Boolean(fixturePaths.human && fixturePaths.machine),
      };
    });

    report.issue = {
      number: issue.number,
      title: issue.title,
      html_url: issue.html_url,
      updated_at: issue.updated_at,
    };
    report.summary = {
      total: entries.length,
      with_human_fixture: entries.filter((entry) => entry.fixture_paths.human).length,
      with_machine_fixture: entries.filter((entry) => entry.fixture_paths.machine).length,
      by_section: countBy(entries, (entry) => entry.section),
      by_upstream_state: countBy(entries, (entry) => entry.upstream_state),
    };
    report.entries = entries;
  }

  if (args.json) {
    console.log(JSON.stringify(report, null, 2));
    return;
  }

  printTextReport(report);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
