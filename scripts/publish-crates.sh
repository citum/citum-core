#!/usr/bin/env bash
# Publish all Citum crates to crates.io in dependency order.
#
# Idempotent: each crate's published version on crates.io is checked
# first; already-published versions are skipped. Safe to re-run after
# partial failure (e.g. transient registry errors), or to re-trigger
# the publish job from the GitHub Actions UI without yanking anything.
#
# Required env: CARGO_REGISTRY_TOKEN — set in repo secrets in CI; for
# local runs, paste from https://crates.io/me.
#
# Usage:
#   CARGO_REGISTRY_TOKEN=... ./scripts/publish-crates.sh
#   CARGO_REGISTRY_TOKEN=... ./scripts/publish-crates.sh --dry-run
#   CARGO_REGISTRY_TOKEN=... ./scripts/publish-crates.sh --skip citum
#   CARGO_REGISTRY_TOKEN=... ./scripts/publish-crates.sh --only citum

set -euo pipefail

DRY_RUN=""
ONLY_CRATE=""
SKIP_CRATES=()

usage() {
  cat <<'USAGE'
Usage: scripts/publish-crates.sh [--dry-run] [--only <crate>] [--skip <crate> ...]

Publishes Citum crates to crates.io in dependency order.

Options:
  --dry-run       Run `cargo publish --dry-run --allow-dirty`.
  --only <crate>  Publish only one crate from the ordered list.
  --skip <crate>  Skip a crate from the ordered list. Repeatable.
  -h, --help      Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN="--dry-run --allow-dirty"
      shift
      ;;
    --only)
      if [[ "${2:-}" == "" ]]; then
        echo "error: --only requires a crate name" >&2
        exit 2
      fi
      ONLY_CRATE="$2"
      shift 2
      ;;
    --skip)
      if [[ "${2:-}" == "" ]]; then
        echo "error: --skip requires a crate name" >&2
        exit 2
      fi
      SKIP_CRATES+=("$2")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "${CARGO_REGISTRY_TOKEN:-}" == "" && "$DRY_RUN" == "" ]]; then
  echo "error: CARGO_REGISTRY_TOKEN is not set" >&2
  exit 1
fi

if [[ "$DRY_RUN" != "" ]]; then
  echo "==> Running in dry-run mode (no uploads, dirty tree allowed)"
fi

# Strict dependency order. Each crate must publish before any crate
# that depends on it. Updating this list:
#   1. Add new internal crates here in dep-graph topological order.
#   2. After adding, verify with `cargo tree -p <new-crate>` that no
#      earlier entry in this list depends on the new one (cycle check).
CRATES=(
  citum-edtf
  citum-resolver-api
  csl-legacy
  citum-schema-data
  citum-schema-style
  citum-schema
  citum-engine
  citum-io
  citum_store
  citum-migrate
  citum-server
  citum
)

contains_crate() {
  local needle="$1"
  shift
  local crate
  for crate in "$@"; do
    if [[ "$crate" == "$needle" ]]; then
      return 0
    fi
  done
  return 1
}

if [[ "$ONLY_CRATE" != "" && ${#SKIP_CRATES[@]} -gt 0 ]]; then
  echo "error: --only cannot be combined with --skip" >&2
  exit 2
fi

if [[ "$ONLY_CRATE" != "" ]] && ! contains_crate "$ONLY_CRATE" "${CRATES[@]}"; then
  echo "error: unknown crate for --only: $ONLY_CRATE" >&2
  exit 2
fi

if [[ ${#SKIP_CRATES[@]} -gt 0 ]]; then
  for skipped in "${SKIP_CRATES[@]}"; do
    if ! contains_crate "$skipped" "${CRATES[@]}"; then
      echo "error: unknown crate for --skip: $skipped" >&2
      exit 2
    fi
  done
fi

SELECTED_CRATES=()
for crate in "${CRATES[@]}"; do
  if [[ "$ONLY_CRATE" != "" && "$crate" != "$ONLY_CRATE" ]]; then
    continue
  fi
  if [[ ${#SKIP_CRATES[@]} -gt 0 ]] && contains_crate "$crate" "${SKIP_CRATES[@]}"; then
    continue
  fi
  SELECTED_CRATES+=("$crate")
done

if [[ ${#SELECTED_CRATES[@]} -eq 0 ]]; then
  echo "error: no crates selected" >&2
  exit 2
fi

echo "==> Selected crates: ${SELECTED_CRATES[*]}"

# Sparse-index propagation delay between publishes. After a successful
# `cargo publish`, the new version takes a few seconds to appear in the
# sparse index that the next crate's `cargo publish` will query. 10s is
# generous; 5s usually works. Skipped for the last crate (no dependent).
PROPAGATION_DELAY=10

published_version_for() {
  # Query crates.io for the highest published version of a crate.
  # Empty string if the crate doesn't exist or has no versions.
  local crate="$1"
  curl --silent --fail "https://crates.io/api/v1/crates/${crate}" \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("crate",{}).get("max_version",""))' \
    2>/dev/null || echo ""
}

local_version_for() {
  # Read [package].version (or workspace inherited) for a crate.
  local crate="$1"
  cargo metadata --no-deps --format-version 1 \
    | python3 -c "import json,sys; m=json.load(sys.stdin); print(next(p['version'] for p in m['packages'] if p['name']=='${crate}'))"
}

is_expected_dry_run_dependency_gap() {
  local output_file="$1"
  local crate
  for crate in "${CRATES[@]}"; do
    if grep -F "no matching package named \`$crate\` found" "$output_file" >/dev/null; then
      return 0
    fi
  done
  return 1
}

for idx in "${!SELECTED_CRATES[@]}"; do
  crate="${SELECTED_CRATES[$idx]}"
  local_ver=$(local_version_for "$crate")
  published_ver=$(published_version_for "$crate")

  echo ""
  echo "==> $crate"
  echo "    local:     $local_ver"
  echo "    published: ${published_ver:-<not on crates.io>}"

  if [[ "$published_ver" == "$local_ver" ]]; then
    echo "    skip: already published at $local_ver"
    continue
  fi

  echo "    publishing ${crate}@${local_ver}..."
  output_file=$(mktemp)
  # shellcheck disable=SC2086 # DRY_RUN is intentionally word-split
  if cargo publish -p "$crate" --locked $DRY_RUN 2>&1 | tee "$output_file"; then
    rm -f "$output_file"
  else
    status=$?
    if [[ "$DRY_RUN" != "" ]] && is_expected_dry_run_dependency_gap "$output_file"; then
      echo "    dry-run note: crates.io has not seen every internal dependency yet."
      echo "    This is expected before the first ordered publish; real publish mode still fails here."
      rm -f "$output_file"
      continue
    fi
    rm -f "$output_file"
    exit "$status"
  fi

  last_idx=$(( ${#SELECTED_CRATES[@]} - 1 ))
  if [[ "$idx" -ne "$last_idx" && "$DRY_RUN" == "" ]]; then
    echo "    waiting ${PROPAGATION_DELAY}s for sparse index..."
    sleep "$PROPAGATION_DELAY"
  fi
done

echo ""
echo "==> Done. Run \`cargo owner --add github:citum:cargo-release <crate>\` for any newly-published crate."
