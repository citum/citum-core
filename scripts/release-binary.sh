#!/usr/bin/env bash
# Build the citum + citum-server binaries for one target triple, pack
# them into a release tarball, and emit a SHA256 for the tarball.
#
# Called per-target from the release.yml build matrix. Outputs land at
# ./release-out/<target>/citum-<version>-<target>.tar.gz plus a sibling
# .sha256 file containing one `<hash>  <tarball-name>` line (sha256sum
# format) ready for aggregation into the release-wide SHA256SUMS.
#
# Usage (with explicit args; mirrors how the workflow invokes it):
#   ./scripts/release-binary.sh <target> <version>
# Example:
#   ./scripts/release-binary.sh x86_64-unknown-linux-musl v0.51.0
#
# Env overrides:
#   USE_CROSS=1   force `cross build` (musl on linux, aarch64-cross).
#   OUT_DIR=path  override ./release-out.

set -euo pipefail

TARGET="${1:?usage: release-binary.sh <target> <version>}"
VERSION="${2:?usage: release-binary.sh <target> <version>}"
OUT_DIR="${OUT_DIR:-release-out}"
USE_CROSS="${USE_CROSS:-0}"

# Strip the leading `v` from the version for the tarball name; this
# matches the on-disk filename convention many install scripts assume.
VER_BARE="${VERSION#v}"
STAGE_DIR="citum-${VER_BARE}-${TARGET}"
TARBALL_NAME="${STAGE_DIR}.tar.gz"

# Windows binaries get .exe; everything else is bare.
EXE_SUFFIX=""
case "$TARGET" in
  *-pc-windows-*) EXE_SUFFIX=".exe" ;;
esac

# Build. `cross` for musl + aarch64-linux (host-tool mismatch); cargo for
# all native targets. `--locked` is required (CI shouldn't update the
# lockfile mid-release).
echo "==> Building citum + citum-server for ${TARGET}"
if [[ "$USE_CROSS" == "1" ]]; then
  cross build --release --locked --target "$TARGET" --bin citum --bin citum-server
else
  rustup target add "$TARGET" 2>/dev/null || true
  cargo build --release --locked --target "$TARGET" --bin citum --bin citum-server
fi

# Stage tarball contents in a dedicated dir; the dir-name becomes the
# top-level directory when users `tar xzf`.
STAGE_PATH="${OUT_DIR}/${TARGET}/${STAGE_DIR}"
mkdir -p "$STAGE_PATH"
cp "target/${TARGET}/release/citum${EXE_SUFFIX}"        "$STAGE_PATH/"
cp "target/${TARGET}/release/citum-server${EXE_SUFFIX}" "$STAGE_PATH/"
cp README.md "$STAGE_PATH/"
# Ship both licenses if present; otherwise a single LICENSE file works.
for f in LICENSE LICENSE-MIT LICENSE-APACHE LICENSE.txt; do
  [[ -f "$f" ]] && cp "$f" "$STAGE_PATH/" || true
done

# Strip the binaries to keep tarballs small (no debug info; ~3-5x smaller).
# Best-effort: not every build env has strip available.
case "$TARGET" in
  *-apple-darwin)
    strip "${STAGE_PATH}/citum${EXE_SUFFIX}"        2>/dev/null || true
    strip "${STAGE_PATH}/citum-server${EXE_SUFFIX}" 2>/dev/null || true
    ;;
  *-pc-windows-*)
    # MSVC strip isn't relevant; binaries built without debug info.
    ;;
  *)
    strip --strip-unneeded "${STAGE_PATH}/citum${EXE_SUFFIX}"        2>/dev/null || true
    strip --strip-unneeded "${STAGE_PATH}/citum-server${EXE_SUFFIX}" 2>/dev/null || true
    ;;
esac

# Tar it up.
TARBALL_PATH="${OUT_DIR}/${TARGET}/${TARBALL_NAME}"
echo "==> Packaging ${TARBALL_NAME}"
tar -czf "$TARBALL_PATH" -C "${OUT_DIR}/${TARGET}" "$STAGE_DIR"

# Emit the SHA256 in sha256sum(1) format. The release job concats every
# per-target .sha256 into one SHA256SUMS attached to the GitHub Release.
SHA256_PATH="${TARBALL_PATH}.sha256"
if command -v sha256sum >/dev/null; then
  ( cd "${OUT_DIR}/${TARGET}" && sha256sum "$TARBALL_NAME" > "${TARBALL_NAME}.sha256" )
else
  # macOS native shasum.
  ( cd "${OUT_DIR}/${TARGET}" && shasum -a 256 "$TARBALL_NAME" > "${TARBALL_NAME}.sha256" )
fi

echo "==> Done: ${TARBALL_PATH}"
echo "    $(cat "$SHA256_PATH")"
