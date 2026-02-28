const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const {
  cleanupOracleTempWorkspace,
  createOracleTempWorkspace,
} = require('./oracle');

const projectRoot = path.resolve(__dirname, '..');
const oracleScript = path.join(__dirname, 'oracle.js');

function runOracle(stylePath) {
  return new Promise((resolve, reject) => {
    const proc = spawn('node', [oracleScript, stylePath, '--json'], {
      cwd: projectRoot,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    proc.stdout.on('data', (chunk) => {
      stdout += chunk;
    });

    proc.stderr.on('data', (chunk) => {
      stderr += chunk;
    });

    proc.on('error', reject);
    proc.on('close', (code) => {
      if (code === 2) {
        reject(new Error(`oracle fatal for ${stylePath}: ${stderr || stdout}`));
        return;
      }

      try {
        resolve({
          code,
          json: JSON.parse(stdout),
          stderr,
        });
      } catch (error) {
        reject(new Error(`failed to parse oracle output for ${stylePath}: ${error.message}\n${stdout}\n${stderr}`));
      }
    });
  });
}

test('oracle temp workspaces are unique and removable', () => {
  const first = createOracleTempWorkspace();
  const second = createOracleTempWorkspace();

  assert.notEqual(first.dir, second.dir);
  assert.equal(fs.existsSync(first.dir), true);
  assert.equal(fs.existsSync(second.dir), true);

  cleanupOracleTempWorkspace(first);
  cleanupOracleTempWorkspace(second);

  assert.equal(fs.existsSync(first.dir), false);
  assert.equal(fs.existsSync(second.dir), false);
});

test('parallel oracle invocations do not collide on temp files', { timeout: 240000 }, async () => {
  const styles = [
    path.join(projectRoot, 'styles-legacy', 'american-association-for-cancer-research.csl'),
    path.join(projectRoot, 'styles-legacy', 'american-institute-of-physics.csl'),
  ];

  const results = await Promise.all(styles.map((stylePath) => runOracle(stylePath)));

  for (let i = 0; i < results.length; i += 1) {
    const stylePath = styles[i];
    const expectedStyle = path.basename(stylePath, '.csl');
    const result = results[i];

    assert.ok([0, 1].includes(result.code), `unexpected oracle exit code for ${expectedStyle}: ${result.code}`);
    assert.equal(result.json.style, expectedStyle);
    assert.equal(result.json.error, undefined, `oracle reported fatal error for ${expectedStyle}`);
  }

  for (const tempFile of ['.migrated-refs.json', '.migrated-citations.json', '.migrated-temp.yaml']) {
    assert.equal(
      fs.existsSync(path.join(projectRoot, tempFile)),
      false,
      `legacy shared temp file should not exist: ${tempFile}`
    );
  }
});
