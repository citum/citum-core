#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ONLY=false
COPILOT_STRICT=false

usage() {
  cat <<'USAGE'
Usage: scripts/validate-frontmatter.sh [--repo-only] [--copilot-strict]

Validates YAML frontmatter for local Claude skills/commands.

Options:
  --repo-only      Validate repo-managed manifests only (.claude/skills/*/SKILL.md)
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

if ! command -v ruby >/dev/null 2>&1; then
  echo "Error: Ruby is required for scripts/validate-frontmatter.sh." >&2
  echo "Install Ruby or run in an environment with Ruby in PATH." >&2
  exit 1
fi

files=()
while IFS= read -r path; do
  files+=("$path")
done < <(find "$REPO_ROOT/.claude/skills" -mindepth 2 -maxdepth 2 -type f -name 'SKILL.md' 2>/dev/null | sort)

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

ruby - "$COPILOT_STRICT" "${files[@]}" <<'RUBY'
require 'yaml'

copilot_strict = ARGV.shift == 'true'
files = ARGV
errors = []

files.each do |file|
  text = File.read(file, encoding: 'UTF-8')

  unless text.start_with?("---\n")
    errors << "#{file}:1 missing frontmatter start delimiter (---)"
    next
  end

  match = text.match(/\A---\n(.*?)\n---\n?/m)
  if match.nil?
    errors << "#{file}:1 malformed frontmatter delimiters"
    next
  end

  frontmatter = match[1]
  begin
    parsed = YAML.safe_load(frontmatter, permitted_classes: [], aliases: false)
  rescue Psych::SyntaxError => e
    line = e.line ? e.line + 1 : 1
    errors << "#{file}:#{line} YAML parse error: #{e.problem}"
    next
  end

  unless parsed.is_a?(Hash)
    errors << "#{file}:1 frontmatter must be a YAML mapping"
    next
  end

  %w[name description].each do |key|
    value = parsed[key]
    if !value.is_a?(String) || value.strip.empty?
      errors << "#{file}:1 required key '#{key}' must be a non-empty string"
    end
  end

  next unless copilot_strict

  frontmatter.each_line.with_index(2) do |line, line_no|
    stripped = line.strip
    next if stripped.empty? || stripped.start_with?('#')

    key_value = stripped.match(/^([A-Za-z0-9_-]+):\s*(.+)$/)
    next unless key_value

    value = key_value[2].strip
    next if value.empty?
    next if value.start_with?("'", '"', '[', '{', '|', '>', '&', '*', '!')

    risky_characters = value.match?(/[:\[\]\(\)\|]/)
    risky_options = value.match?(/\[[^\]]+\]/) || value.match?(/--[A-Za-z0-9_-]+/) || value.match?(/\s+or\s+/)

    if risky_characters || risky_options
      errors << "#{file}:#{line_no} copilot-strict: quote risky plain scalar for '#{key_value[1]}'"
    end
  end
end

if errors.empty?
  puts "Frontmatter validation passed for #{files.length} file(s)."
  exit 0
end

puts "Frontmatter validation failed (#{errors.length} issue#{errors.length == 1 ? '' : 's'}):"
errors.each { |err| puts "- #{err}" }
exit 1
RUBY
