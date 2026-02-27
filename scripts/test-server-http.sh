#!/usr/bin/env bash

set -euo pipefail

SHOW_OUTPUT=0
SUMMARY_ONLY=1
PORT=8080

usage() {
  cat <<'EOF'
Usage: ./scripts/test-server-http.sh [--show-output] [--output-only] [--port PORT] [port]

Options:
  --show-output  Print each JSON response as well as PASS/FAIL checks
  --output-only  Print each JSON response without PASS lines
  --port PORT    Use a specific port
  -h, --help     Show this help text
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --show-output)
      SHOW_OUTPUT=1
      shift
      ;;
    --output-only)
      SHOW_OUTPUT=1
      SUMMARY_ONLY=0
      shift
      ;;
    --port)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --port" >&2
        exit 1
      fi
      PORT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      PORT="$1"
      shift
      ;;
  esac
done

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STYLE_PATH="$ROOT_DIR/styles/apa-7th.yaml"
RPC_URL="http://127.0.0.1:${PORT}/rpc"

assert_contains() {
  local body="$1"
  local filter="$2"
  local label="$3"

  if ! printf '%s' "$body" | jq -e "$filter" >/dev/null; then
    echo "FAIL: ${label}" >&2
    echo "Failed jq filter: ${filter}" >&2
    echo "Actual response: ${body}" >&2
    exit 1
  fi

  if [[ "$SUMMARY_ONLY" -eq 1 ]]; then
    echo "PASS: ${label}"
  fi
}

post_json() {
  local payload="$1"
  curl -sS "$RPC_URL" \
    -H 'Content-Type: application/json' \
    -d "$payload"
}

print_response() {
  local label="$1"
  local body="$2"

  if [[ "$SHOW_OUTPUT" -eq 1 ]]; then
    echo
    echo "[${label}]"
    printf '%s' "$body" | jq .
  fi
}

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT

cargo run -p citum-server --features http -- --http --port "$PORT" &
SERVER_PID=$!

for _ in {1..40}; do
  if curl -sS -o /dev/null "$RPC_URL"; then
    break
  fi
  sleep 0.25
done

echo "Running HTTP smoke tests against ${RPC_URL}"

COMMON_REFS='{
  "ITEM-2": {
    "id": "ITEM-2",
    "class": "monograph",
    "type": "book",
    "title": "A Brief History of Time",
    "author": [{"family": "Hawking", "given": "Stephen"}],
    "issued": "1988"
  }
}'

render_citation_response="$(post_json "{
  \"id\": 1,
  \"method\": \"render_citation\",
  \"params\": {
    \"style_path\": \"${STYLE_PATH}\",
    \"refs\": ${COMMON_REFS},
    \"citation\": {
      \"id\": \"cite-1\",
      \"items\": [{\"id\": \"ITEM-2\"}]
    }
  }
}")"
print_response "render_citation" "$render_citation_response"
assert_contains "$render_citation_response" '.id == 1' 'render_citation returns request id'
assert_contains "$render_citation_response" '.result | type == "string"' 'render_citation returns string result'
assert_contains "$render_citation_response" '.result | test("Hawking|1988")' 'render_citation returns citation content'

render_bibliography_response="$(post_json "{
  \"id\": 2,
  \"method\": \"render_bibliography\",
  \"params\": {
    \"style_path\": \"${STYLE_PATH}\",
    \"refs\": ${COMMON_REFS}
  }
}")"
print_response "render_bibliography" "$render_bibliography_response"
assert_contains "$render_bibliography_response" '.id == 2' 'render_bibliography returns request id'
assert_contains "$render_bibliography_response" '.result | type == "array" and length > 0' 'render_bibliography returns entries'
assert_contains "$render_bibliography_response" '.result | join("\n") | test("A Brief History of Time")' 'render_bibliography returns book title'

validate_style_response="$(post_json "{
  \"id\": 3,
  \"method\": \"validate_style\",
  \"params\": {
    \"style_path\": \"${STYLE_PATH}\"
  }
}")"
print_response "validate_style" "$validate_style_response"
assert_contains "$validate_style_response" '.id == 3' 'validate_style returns request id'
assert_contains "$validate_style_response" '.result.valid == true' 'validate_style reports valid style'

unknown_method_response="$(post_json "{
  \"id\": 4,
  \"method\": \"frobnicate\",
  \"params\": {}
}")"
print_response "unknown_method" "$unknown_method_response"
assert_contains "$unknown_method_response" '.id == 4' 'unknown method returns request id'
assert_contains "$unknown_method_response" '.error | contains("unknown method")' 'unknown method returns error message'

missing_field_response="$(post_json "{
  \"id\": 5,
  \"method\": \"render_bibliography\",
  \"params\": {}
}")"
print_response "missing_field" "$missing_field_response"
assert_contains "$missing_field_response" '.id == 5' 'missing field returns request id'
assert_contains "$missing_field_response" '.error | contains("style_path")' 'missing field reports required param'

echo "HTTP smoke tests passed."
