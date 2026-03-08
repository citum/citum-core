#!/usr/bin/env bash
# scripts/bench-check.sh: capture and compare local benchmark baselines.

set -euo pipefail

BASELINE_DIR=".bench-baselines"
BENCH_TARGETS=("rendering" "formats")
BENCH_FILES=(
  "crates/citum-engine/benches/rendering.rs"
  "crates/citum-schema/benches/formats.rs"
)

usage() {
  cat <<'EOF'
Usage:
  ./scripts/bench-check.sh capture <name>
  ./scripts/bench-check.sh compare <baseline> <current>

Compatibility forms:
  ./scripts/bench-check.sh <name>              # capture
  ./scripts/bench-check.sh <baseline> <current> # compare
  ./scripts/bench-check.sh                     # compare baseline vs current

The default benchmark set is:
  - cargo bench --bench rendering
  - cargo bench --bench formats
EOF
}

say() {
  printf '%s\n' "$*"
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

ensure_dir() {
  mkdir -p "$BASELINE_DIR"
}

ensure_bench_targets() {
  local missing=0
  local path

  for path in "${BENCH_FILES[@]}"; do
    if [[ ! -f "$path" ]]; then
      printf 'Missing benchmark target file: %s\n' "$path" >&2
      missing=1
    fi
  done

  if [[ "$missing" -ne 0 ]]; then
    fail "benchmark target set is out of sync with the repo"
  fi
}

run_benches() {
  local output_file=$1
  local target

  say "Writing benchmark output to $output_file"
  {
    for target in "${BENCH_TARGETS[@]}"; do
      say "--- cargo bench --bench $target ---" >&2
      cargo bench --bench "$target"
    done
  } >"$output_file"
}

capture_baseline() {
  local name=$1
  local output_file="$BASELINE_DIR/$name.txt"

  ensure_dir
  ensure_bench_targets

  say "--- Capturing benchmark baseline: $name ---"
  run_benches "$output_file"
  say "Done. Captured $name at $output_file"
}

compare_baselines() {
  local baseline_name=$1
  local current_name=$2
  local baseline_file="$BASELINE_DIR/$baseline_name.txt"
  local current_file="$BASELINE_DIR/$current_name.txt"

  ensure_dir
  ensure_bench_targets

  if [[ ! -f "$baseline_file" ]]; then
    fail "no baseline file found at $baseline_file. Capture one first with './scripts/bench-check.sh capture $baseline_name'"
  fi

  if ! command -v critcmp >/dev/null 2>&1; then
    fail "'critcmp' not found. Install it with: cargo install critcmp"
  fi

  say "--- Benchmarking current state: $current_name ---"
  run_benches "$current_file"

  say ""
  say "--- Performance delta: $baseline_name vs $current_name ---"
  critcmp "$baseline_file" "$current_file"
}

main() {
  case "${1:-}" in
    "" )
      compare_baselines "baseline" "current"
      ;;
    capture)
      [[ $# -eq 2 ]] || fail "capture requires exactly one name"
      capture_baseline "$2"
      ;;
    compare)
      [[ $# -eq 3 ]] || fail "compare requires a baseline name and current name"
      compare_baselines "$2" "$3"
      ;;
    -h|--help|help)
      usage
      ;;
    *)
      if [[ $# -eq 1 ]]; then
        capture_baseline "$1"
      elif [[ $# -eq 2 ]]; then
        compare_baselines "$1" "$2"
      else
        usage >&2
        exit 1
      fi
      ;;
  esac
}

main "$@"
