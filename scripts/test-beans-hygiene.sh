#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
WRAPPER="$ROOT_DIR/.claude/skills/beans/bin/citum-bean"
HYGIENE_SCRIPT="$ROOT_DIR/scripts/check-docs-beans-hygiene.sh"
TMP_ROOT=$(mktemp -d)

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

assert_contains() {
  local haystack=$1
  local needle=$2

  if [[ "$haystack" != *"$needle"* ]]; then
    fail "expected output to contain: $needle"
  fi
}

assert_not_contains() {
  local haystack=$1
  local needle=$2

  if [[ "$haystack" == *"$needle"* ]]; then
    fail "expected output not to contain: $needle"
  fi
}

new_repo() {
  local name=$1
  local repo="$TMP_ROOT/$name"
  mkdir -p "$repo/.beans" "$repo/bin"
  git -C "$repo" init -q -b main
  git -C "$repo" config user.name "Test User"
  git -C "$repo" config user.email "test@example.com"
  cat >"$repo/bin/beans" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  list)
    if [[ "${2:-}" != "--json" ]]; then
      echo "unexpected beans args: $*" >&2
      exit 1
    fi
    cat "$BEANS_LIST_JSON"
    ;;
  *)
    echo "unsupported beans command: $*" >&2
    exit 1
    ;;
esac
EOF
  chmod +x "$repo/bin/beans"
  printf '%s\n' "$repo"
}

commit_file() {
  local repo=$1
  local file=$2
  local content=$3
  local message=$4

  mkdir -p "$(dirname "$repo/$file")"
  printf '%s\n' "$content" >"$repo/$file"
  git -C "$repo" add "$file"
  git -C "$repo" commit -q -m "$message"
}

run_wrapper() {
  local repo=$1
  local beans_json=$2
  shift 2
  (
    cd "$repo"
    BEANS_LIST_JSON="$beans_json" PATH="$repo/bin:$PATH" "$WRAPPER" "$@"
  )
}

run_hygiene_script() {
  local repo=$1
  local wrapper=$2
  local script_copy="$repo/scripts/check-docs-beans-hygiene.sh"
  mkdir -p "$repo/scripts"
  cp "$HYGIENE_SCRIPT" "$script_copy"
  chmod +x "$script_copy"
  (
    cd "$repo"
    BEAN_WRAPPER="$wrapper" "$script_copy"
  )
}

cat >"$TMP_ROOT/no-match.json" <<'EOF'
[
  {
    "id": "csl26-none",
    "title": "Unrelated task",
    "status": "todo",
    "type": "task",
    "priority": "normal",
    "path": "csl26-none--unrelated-task.md"
  }
]
EOF

repo=$(new_repo no-match)
commit_file "$repo" README.md "baseline" "chore: initial commit"
no_match_output=$(run_wrapper "$repo" "$TMP_ROOT/no-match.json" hygiene)
assert_contains "$no_match_output" "No bean hygiene issues found."

cat >"$TMP_ROOT/hard-stale.json" <<'EOF'
[
  {
    "id": "csl26-stal",
    "title": "Investigate stale bean detection",
    "status": "todo",
    "type": "task",
    "priority": "normal",
    "path": "csl26-stal--investigate-stale-bean-detection.md"
  }
]
EOF

repo=$(new_repo hard-stale)
commit_file "$repo" README.md "baseline" "chore: initial commit"
commit_file "$repo" notes.txt "done" $'fix(workflow): complete csl26-stal\n\nRefs: csl26-stal'
set +e
hard_output=$(run_wrapper "$repo" "$TMP_ROOT/hard-stale.json" hygiene 2>&1)
hard_status=$?
set -e
[[ "$hard_status" -eq 1 ]] || fail "expected hard stale hygiene to fail"
assert_contains "$hard_output" "Open beans with landed work on main (hard failure):"
assert_contains "$hard_output" "evidence: bean-id"
assert_contains "$hard_output" "fix(workflow): complete csl26-stal"

cat >"$TMP_ROOT/soft-stale.json" <<'EOF'
[
  {
    "id": "csl26-soft",
    "title": "Investigate stale bean detection",
    "status": "todo",
    "type": "task",
    "priority": "normal",
    "path": "csl26-soft--investigate-stale-bean-detection.md"
  }
]
EOF

repo=$(new_repo soft-stale)
commit_file "$repo" README.md "baseline" "chore: initial commit"
commit_file "$repo" notes.txt "done" "Investigate stale bean detection"
soft_output=$(run_wrapper "$repo" "$TMP_ROOT/soft-stale.json" hygiene)
assert_contains "$soft_output" "Open beans with advisory title-only matches on main:"
assert_contains "$soft_output" "[unique-title-match]"

cat >"$TMP_ROOT/root-terminal.json" <<'EOF'
[
  {
    "id": "csl26-root",
    "title": "Archive me",
    "status": "completed",
    "type": "task",
    "priority": "normal",
    "path": "csl26-root--archive-me.md"
  }
]
EOF

repo=$(new_repo root-terminal)
commit_file "$repo" README.md "baseline" "chore: initial commit"
set +e
root_output=$(run_wrapper "$repo" "$TMP_ROOT/root-terminal.json" hygiene 2>&1)
root_status=$?
set -e
[[ "$root_status" -eq 1 ]] || fail "expected root terminal hygiene to fail"
assert_contains "$root_output" "Root-level terminal beans that should be archived:"

cat >"$TMP_ROOT/archived-terminal.json" <<'EOF'
[
  {
    "id": "csl26-arch",
    "title": "Archived bean",
    "status": "completed",
    "type": "task",
    "priority": "normal",
    "path": "archive/csl26-arch--archived-bean.md"
  }
]
EOF

repo=$(new_repo archived-terminal)
commit_file "$repo" README.md "baseline" "chore: initial commit"
archived_output=$(run_wrapper "$repo" "$TMP_ROOT/archived-terminal.json" hygiene)
assert_contains "$archived_output" "No bean hygiene issues found."

cat >"$TMP_ROOT/title-collision.json" <<'EOF'
[
  {
    "id": "csl26-open",
    "title": "Shared title",
    "status": "todo",
    "type": "task",
    "priority": "normal",
    "path": "csl26-open--shared-title.md"
  },
  {
    "id": "csl26-done",
    "title": "Shared title",
    "status": "completed",
    "type": "task",
    "priority": "normal",
    "path": "archive/csl26-done--shared-title.md"
  }
]
EOF

repo=$(new_repo title-collision)
commit_file "$repo" README.md "baseline" "chore: initial commit"
set +e
collision_output=$(run_wrapper "$repo" "$TMP_ROOT/title-collision.json" hygiene 2>&1)
collision_status=$?
set -e
[[ "$collision_status" -eq 1 ]] || fail "expected title collision hygiene to fail"
assert_contains "$collision_output" "Open beans colliding with completed/scrapped titles:"

mock_wrapper_ok="$TMP_ROOT/mock-wrapper-ok.sh"
cat >"$mock_wrapper_ok" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "No bean hygiene issues found."
EOF
chmod +x "$mock_wrapper_ok"

repo="$TMP_ROOT/docs-only"
mkdir -p "$repo/docs"
cat >"$repo/docs/broken.md" <<'EOF'
[Broken](./missing.md)
EOF
set +e
docs_only_output=$(run_hygiene_script "$repo" "$mock_wrapper_ok" 2>&1)
docs_only_status=$?
set -e
[[ "$docs_only_status" -eq 1 ]] || fail "expected docs-only hygiene script to fail"
assert_contains "$docs_only_output" "[check] markdown relative links"
assert_contains "$docs_only_output" "ERROR: broken markdown links found"
assert_not_contains "$docs_only_output" "ERROR: bean hygiene failed"

mock_wrapper_fail="$TMP_ROOT/mock-wrapper-fail.sh"
cat >"$mock_wrapper_fail" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "Bean audit scope: open"
echo
echo "Open beans with landed work on main (hard failure):"
echo "  - csl26-stal [todo] Investigate stale bean detection (csl26-stal.md)"
exit 1
EOF
chmod +x "$mock_wrapper_fail"

repo="$TMP_ROOT/bean-only"
mkdir -p "$repo/docs"
cat >"$repo/docs/ok.md" <<'EOF'
[Self](./ok.md)
EOF
set +e
bean_only_output=$(run_hygiene_script "$repo" "$mock_wrapper_fail" 2>&1)
bean_only_status=$?
set -e
[[ "$bean_only_status" -eq 1 ]] || fail "expected bean-only hygiene script to fail"
assert_contains "$bean_only_output" "ok: no broken markdown links"
assert_contains "$bean_only_output" "ERROR: bean hygiene failed"
assert_contains "$bean_only_output" "Open beans with landed work on main (hard failure):"

repo="$TMP_ROOT/combined"
mkdir -p "$repo/docs"
cat >"$repo/docs/broken.md" <<'EOF'
[Broken](./missing.md)
EOF
set +e
combined_output=$(run_hygiene_script "$repo" "$mock_wrapper_fail" 2>&1)
combined_status=$?
set -e
[[ "$combined_status" -eq 1 ]] || fail "expected combined hygiene script to fail"
assert_contains "$combined_output" "ERROR: broken markdown links found"
assert_contains "$combined_output" "ERROR: bean hygiene failed"
assert_contains "$combined_output" "hygiene check failed with 2 error(s)"

printf 'test-beans-hygiene.sh: ok\n'
