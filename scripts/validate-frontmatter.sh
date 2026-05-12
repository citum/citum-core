#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ONLY=false
COPILOT_STRICT=false

usage() {
  cat <<'USAGE'
Usage: scripts/validate-frontmatter.sh [--repo-only] [--copilot-strict]

Validates YAML frontmatter for local skills/commands.

Options:
  --repo-only      Validate repo-managed manifests only (.claude/skills/*/SKILL.md and .skills/*/SKILL.md)
  --copilot-strict Reject risky unquoted plain scalars that often break Copilot parsing
  -h, --help       Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-only)
      REPO_ONLY=true
      shift
      ;;
    --copilot-strict)
      COPILOT_STRICT=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v node >/dev/null 2>&1; then
  echo "Error: Node.js is required for scripts/validate-frontmatter.sh." >&2
  exit 1
fi

files=()
while IFS= read -r path; do
  files+=("$path")
done < <(find "$REPO_ROOT/.claude/skills" -mindepth 2 -maxdepth 2 -type f -name 'SKILL.md' 2>/dev/null | sort)

while IFS= read -r path; do
  files+=("$path")
done < <(find "$REPO_ROOT/.skills" -mindepth 2 -maxdepth 2 -type f -name 'SKILL.md' 2>/dev/null | sort)

if [[ "$REPO_ONLY" == false ]]; then
  while IFS= read -r path; do
    files+=("$path")
  done < <(find "$HOME/.claude/skills" -mindepth 2 -maxdepth 2 -type f -name 'SKILL.md' 2>/dev/null | sort)

  while IFS= read -r path; do
    files+=("$path")
  done < <(find "$HOME/.claude/commands" -maxdepth 1 -type f -name '*.md' 2>/dev/null | sort)
fi

if [[ ${#files[@]} -eq 0 ]]; then
  echo "No manifest files found."
  exit 0
fi

SCRIPTS_DIR="$REPO_ROOT/scripts" node - "$COPILOT_STRICT" "${files[@]}" <<'JS'
const fs = require('fs');
const path = require('path');
const yaml = require(path.join(process.env.SCRIPTS_DIR, 'node_modules/js-yaml'));

const [,, copilotStrictArg, ...files] = process.argv;
const copilotStrict = copilotStrictArg === 'true';
const errors = [];

for (const file of files) {
  const text = fs.readFileSync(file, 'utf8');

  if (!text.startsWith('---\n')) {
    errors.push(`${file}:1 missing frontmatter start delimiter (---)`);
    continue;
  }

  const match = text.match(/^---\n([\s\S]*?)\n---\n?/);
  if (!match) {
    errors.push(`${file}:1 malformed frontmatter delimiters`);
    continue;
  }

  const frontmatter = match[1];
  let parsed;
  try {
    parsed = yaml.load(frontmatter, { schema: yaml.CORE_SCHEMA });
  } catch (e) {
    const line = e.mark ? e.mark.line + 2 : 1;
    errors.push(`${file}:${line} YAML parse error: ${e.reason || e.message}`);
    continue;
  }

  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    errors.push(`${file}:1 frontmatter must be a YAML mapping`);
    continue;
  }

  for (const key of ['name', 'description']) {
    const value = parsed[key];
    if (typeof value !== 'string' || value.trim() === '') {
      errors.push(`${file}:1 required key '${key}' must be a non-empty string`);
    }
  }

  if (!copilotStrict) continue;

  const lines = frontmatter.split('\n');
  lines.forEach((line, idx) => {
    const lineNo = idx + 2;
    const stripped = line.trim();
    if (!stripped || stripped.startsWith('#')) return;

    const kv = stripped.match(/^([A-Za-z0-9_-]+):\s*(.+)$/);
    if (!kv) return;

    const value = kv[2].trim();
    if (!value) return;
    if (/^['"\[{|>&*!]/.test(value)) return;

    const riskyChars = /[:\[\]()|]/.test(value);
    const riskyOpts = /\[[^\]]+\]/.test(value) || /--[A-Za-z0-9_-]+/.test(value) || /\s+or\s+/.test(value);

    if (riskyChars || riskyOpts) {
      errors.push(`${file}:${lineNo} copilot-strict: quote risky plain scalar for '${kv[1]}'`);
    }
  });
}

if (errors.length === 0) {
  console.log(`Frontmatter validation passed for ${files.length} file(s).`);
  process.exit(0);
}

console.log(`Frontmatter validation failed (${errors.length} issue${errors.length === 1 ? '' : 's'}):`);
errors.forEach(err => console.log(`- ${err}`));
process.exit(1);
JS
