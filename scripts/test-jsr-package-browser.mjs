import { createReadStream } from 'node:fs';
import { stat } from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { chromium } from 'playwright';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const mimeTypes = new Map([
  ['.html', 'text/html; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.mjs', 'text/javascript; charset=utf-8'],
  ['.wasm', 'application/wasm'],
]);

const server = http.createServer(async (request, response) => {
  const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
  const relative = pathname === '/' ? '/scripts/jsr-package-browser-smoke.html' : pathname;
  const target = path.resolve(repoRoot, `.${relative}`);
  if (!target.startsWith(`${repoRoot}${path.sep}`)) {
    response.writeHead(403).end();
    return;
  }
  try {
    const metadata = await stat(target);
    if (!metadata.isFile()) throw new Error('not a file');
    response.writeHead(200, {
      'Content-Type': mimeTypes.get(path.extname(target)) ?? 'application/octet-stream',
    });
    createReadStream(target).pipe(response);
  } catch {
    response.writeHead(404).end();
  }
});

await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const address = server.address();
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${address.port}/`);
  await page.waitForFunction(() => window.smokeResult !== undefined);
  const smoke = await page.evaluate(() => window.smokeResult);
  if (!smoke.ok) throw new Error(`${smoke.error}\n${smoke.stack ?? ''}`);
  console.log(`Browser JSR package smoke passed: ${JSON.stringify(smoke.result)}`);
} finally {
  await browser.close();
  await new Promise((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve()))
  );
}
