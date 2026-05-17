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

set -euo pipefail

if [[ "${CARGO_REGISTRY_TOKEN:-}" == "" && "${1:-}" != "--dry-run" ]]; then
  echo "error: CARGO_REGISTRY_TOKEN is not set" >&2
  exit 1
fi

DRY_RUN=""
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN="--dry-run --allow-dirty"
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

for crate in "${CRATES[@]}"; do
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
  # shellcheck disable=SC2086 # DRY_RUN is intentionally word-split
  cargo publish -p "$crate" --locked $DRY_RUN

  last_idx=$(( ${#CRATES[@]} - 1 ))
  if [[ "$crate" != "${CRATES[$last_idx]}" && "$DRY_RUN" == "" ]]; then
    echo "    waiting ${PROPAGATION_DELAY}s for sparse index..."
    sleep "$PROPAGATION_DELAY"
  fi
done

echo ""
echo "==> Done. Run \`cargo owner --add github:citum:cargo-release <crate>\` for any newly-published crate."
