#!/usr/bin/env bash
# scripts/release.sh — bump workspace version and prepare a release commit
# Usage: ./scripts/release.sh <new-version>
# Example: ./scripts/release.sh 0.8.0
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"
CHANGELOG="$ROOT/CHANGELOG.md"

# ── Validate input ────────────────────────────────────────────────────────────
if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <new-version>  (e.g. 0.8.0)" >&2
  exit 1
fi

NEW_VERSION="$1"
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: '$NEW_VERSION' is not a valid semver (expected X.Y.Z)" >&2
  exit 1
fi

# ── Check working tree is clean ───────────────────────────────────────────────
if ! git -C "$ROOT" diff --quiet || ! git -C "$ROOT" diff --cached --quiet; then
  echo "Error: working tree has uncommitted changes — commit or stash first" >&2
  exit 1
fi

# ── Read current version ──────────────────────────────────────────────────────
CURRENT_VERSION=$(grep -m1 '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')
echo "Bumping $CURRENT_VERSION → $NEW_VERSION"

# ── Bump Cargo.toml ───────────────────────────────────────────────────────────
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

# ── Update CHANGELOG.md ───────────────────────────────────────────────────────
TODAY=$(date +%Y-%m-%d)

if ! grep -q '## \[Unreleased\]' "$CHANGELOG"; then
  echo "Error: CHANGELOG.md missing '## [Unreleased]' section" >&2
  exit 1
fi

# Promote [Unreleased] → [NEW_VERSION] - TODAY
sed -i '' "s/## \[Unreleased\]/## [$NEW_VERSION] - $TODAY/" "$CHANGELOG"

# Prepend fresh [Unreleased] section above the new version entry
sed -i '' "s/## \[$NEW_VERSION\]/## [Unreleased]\n\n## [$NEW_VERSION]/" "$CHANGELOG"

# ── Refresh Cargo.lock ────────────────────────────────────────────────────────
echo "Running cargo check to refresh Cargo.lock..."
cargo check --quiet --manifest-path "$CARGO_TOML" 2>&1

# ── Commit ────────────────────────────────────────────────────────────────────
git -C "$ROOT" add -A
git -C "$ROOT" commit -m "chore(release): bump workspace version to $NEW_VERSION"

echo ""
echo "Done. Push when ready:"
echo "  git push origin main"
