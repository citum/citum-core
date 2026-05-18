// Minimal Node client for citum-server.
//
// Run:
//   node client.mjs   # assumes `citum-server` is on PATH
//
// Sends one format_document request and prints the response.

import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { readFileSync } from "node:fs";

const request = JSON.parse(readFileSync(new URL("./format-document-request.json", import.meta.url)));

const server = spawn("citum-server", [], { stdio: ["pipe", "pipe", "inherit"] });
const lines = createInterface({ input: server.stdout });

server.stdin.write(JSON.stringify(request) + "\n");

for await (const line of lines) {
  const response = JSON.parse(line);
  console.log(JSON.stringify(response, null, 2));
  server.stdin.end();
  break;
}
