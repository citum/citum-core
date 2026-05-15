#!/usr/bin/env bash
# PostToolUse hook: nudge toward jcodemunch / rust-analyzer when the agent
# reads or greps Rust source. The rule is documented in CLAUDE.md but the
# agent has historically rationalized it away — this hook puts the reminder
# back into context at the moment of the violation.
#
# Receives the tool call as JSON on stdin. Emits a hookSpecificOutput
# JSON object on stdout with additionalContext to inject a system reminder.
# Silent (no output) on every other tool call.

# Never block the user's tool call from this hook. Errors in jq parsing,
# unexpected stdin shape, or anything else should fall through to exit 0.
set -uo pipefail

input=$(cat || true)

# Bail quietly if jq is missing — never block a tool call from this hook.
if ! command -v jq >/dev/null 2>&1; then
  exit 0
fi

tool_name=$(printf '%s' "$input" | jq -r '.tool_name // empty' 2>/dev/null || true)

case "$tool_name" in
  Read|Grep|Glob) ;;
  *) exit 0 ;;
esac

# Look for a .rs file path in any of the common parameter keys.
target=$(printf '%s' "$input" | jq -r '
  .tool_input
  | (.file_path // .path // .pattern // .glob // "")
' 2>/dev/null || true)

# Match either a literal .rs path or a pattern containing crates/.
case "$target" in
  *.rs|*crates/*|*citum-engine*|*citum-schema*|*citum-migrate*|*csl-legacy*) ;;
  *) exit 0 ;;
esac

# Skip the nudge for the trivial workspace manifest case.
case "$target" in
  */Cargo.toml|*/Cargo.lock) exit 0 ;;
esac

cat <<'JSON'
{
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "Reminder: for Rust symbol lookups in citum-core, prefer jcodemunch (get_symbol / get_symbols / get_repo_outline) over Read/Grep — the index is live (~184 files, 2308 symbols). For type resolution / hover, prefer rust-analyzer. Bash grep is correct for string literals and call-site patterns, not for finding a named symbol. See root CLAUDE.md → Code Search Tool Priority."
  }
}
JSON
