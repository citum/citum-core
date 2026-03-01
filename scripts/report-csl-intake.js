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

function parseArgs(argv) {
  const args = {
    issueUrl: DEFAULT_ISSUE_URL,
    issueFile: null,
    json: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--json") {
      args.json = true;
    } else if (arg === "--issue-file") {
      args.issueFile = argv[i + 1];
      i += 1;
    } else if (arg === "--issue-url") {
      args.issueUrl = argv[i + 1];
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

function printTextReport(report) {
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

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const root = path.resolve(__dirname, "..");
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

  const report = {
    generated_at: new Date().toISOString(),
    issue: {
      number: issue.number,
      title: issue.title,
      html_url: issue.html_url,
      updated_at: issue.updated_at,
    },
    summary: {
      total: entries.length,
      with_human_fixture: entries.filter((entry) => entry.fixture_paths.human).length,
      with_machine_fixture: entries.filter((entry) => entry.fixture_paths.machine).length,
      by_section: countBy(entries, (entry) => entry.section),
      by_upstream_state: countBy(entries, (entry) => entry.upstream_state),
    },
    entries,
  };

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
