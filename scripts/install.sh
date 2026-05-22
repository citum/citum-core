#!/usr/bin/env sh
# Citum installer.
#
#   curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | sh
#
# Detects your platform, downloads the matching release tarball from
# GitHub, verifies its SHA256 against the checksum manifest in the
# same release, and installs the selected Citum binaries to
# $CITUM_INSTALL_DIR (default $HOME/.local/bin).
#
# Environment overrides:
#   CITUM_VERSION       — release tag (default: latest)
#   CITUM_INSTALL_DIR   — install destination (default: $HOME/.local/bin)
#   CITUM_REPO          — GitHub repo (default: citum/citum-core)
#   CITUM_COMPONENTS    — comma-separated subset of {citum, citum-server,
#                         citum-migrate}, or the alias `all` (default: citum)
#
# Examples:
#   curl -fsSL .../install.sh | sh                                  # citum only
#   curl -fsSL .../install.sh | CITUM_COMPONENTS=all sh             # everything
#   curl -fsSL .../install.sh | CITUM_COMPONENTS=citum,citum-migrate sh

set -eu

REPO="${CITUM_REPO:-citum/citum-core}"
VERSION="${CITUM_VERSION:-latest}"
INSTALL_DIR="${CITUM_INSTALL_DIR:-$HOME/.local/bin}"
COMPONENTS_RAW="${CITUM_COMPONENTS:-citum}"

# ---------- helpers ---------------------------------------------------

say()    { printf '%s\n'    "citum-installer: $*"; }
err()    { printf '%s\n' >&2 "citum-installer: error: $*"; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

# Detect platform triple. Maps `uname -s` + `uname -m` to the target
# names produced by scripts/release-binary.sh.
detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)        echo "x86_64-unknown-linux-musl" ;;
        aarch64|arm64)       echo "aarch64-unknown-linux-musl" ;;
        *) err "unsupported Linux arch: $arch (supported: x86_64, aarch64)" ;;
      esac ;;
    Darwin)
      case "$arch" in
        x86_64)
          if command -v sysctl >/dev/null 2>&1 \
            && [ "$(sysctl -in sysctl.proc_translated 2>/dev/null || printf 0)" = "1" ]; then
            echo "aarch64-apple-darwin"
          else
            err "prebuilt Intel macOS binaries are no longer shipped; install from source with: cargo install citum --locked && cargo install citum-server --locked"
          fi ;;
        arm64)               echo "aarch64-apple-darwin" ;;
        *) err "unsupported macOS arch: $arch (supported prebuilt arch: arm64)" ;;
      esac ;;
    MINGW*|MSYS*|CYGWIN*)    echo "x86_64-pc-windows-msvc" ;;
    *) err "unsupported OS: $os (supported: Linux, Darwin, Windows via Git Bash)" ;;
  esac
}

# Resolve "latest" via the GitHub redirect; gives us the actual tag.
resolve_version() {
  if [ "$VERSION" = "latest" ]; then
    # https://api.github.com/.../releases/latest is rate-limited
    # unauthenticated; the redirect URL is not.
    redirect=$(curl -fsSLI -o /dev/null -w '%{url_effective}\n' \
      "https://github.com/${REPO}/releases/latest")
    VERSION="${redirect##*/}"
    [ -z "$VERSION" ] && err "failed to resolve latest version"
  fi
  case "$VERSION" in
    v*) ;;
    *) VERSION="v${VERSION}" ;;
  esac
  echo "$VERSION"
}

# ---------- main ------------------------------------------------------

need_cmd curl
need_cmd install
need_cmd tar
need_cmd uname

# sha256 verification is non-negotiable for a curl|bash installer.
SHASUM=""
if   command -v sha256sum >/dev/null 2>&1; then SHASUM="sha256sum"
elif command -v shasum    >/dev/null 2>&1; then SHASUM="shasum -a 256"
else err "need sha256sum or shasum to verify the download"
fi

# Resolve component selection up front so bad input fails before any
# network traffic. `all` is the only alias; everything else must be an
# explicit binary name.
case "$COMPONENTS_RAW" in
  all) SELECTED="citum citum-server citum-migrate" ;;
  *)   SELECTED=$(printf '%s' "$COMPONENTS_RAW" | tr ',' ' ') ;;
esac
# Normalize any whitespace (spaces, tabs, newlines a stray CI var might
# carry) to single spaces and trim both ends — `tr` handles the squeeze,
# `awk` handles trim portably without bashisms.
SELECTED=$(printf '%s' "$SELECTED" | tr -s '[:space:]' ' ' | awk '{$1=$1; print}')
[ -z "$SELECTED" ] && err "CITUM_COMPONENTS is empty (valid: citum, citum-server, citum-migrate, all)"
for c in $SELECTED; do
  case "$c" in
    citum|citum-server|citum-migrate) ;;
    *) err "unknown component: $c (valid: citum, citum-server, citum-migrate, all)" ;;
  esac
done

TARGET="$(detect_target)"
VERSION="$(resolve_version)"
VER_BARE="${VERSION#v}"

EXE_SUFFIX=""
case "$TARGET" in
  *-pc-windows-*) EXE_SUFFIX=".exe" ;;
esac

TARBALL="citum-${VER_BARE}-${TARGET}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

say "installing citum ${VERSION} for ${TARGET} (components:$(printf ' %s' $SELECTED))"

# Stage in a tempdir that's wiped on exit (even on error).
TMP="$(mktemp -d 2>/dev/null || mktemp -d -t citum-install)"
trap 'rm -rf "$TMP"' EXIT INT TERM

say "downloading ${TARBALL}"
curl --fail --location --silent --show-error \
  --output "${TMP}/${TARBALL}" \
  "${BASE_URL}/${TARBALL}"

say "downloading SHA256SUMS"
curl --fail --location --silent --show-error \
  --output "${TMP}/SHA256SUMS" \
  "${BASE_URL}/SHA256SUMS"

# Verify against the release's checksum manifest. Extract just the line
# for our tarball; verify only that line. This sidesteps shasum's
# behavior of needing every file in SHA256SUMS to be present.
say "verifying checksum"
expected=$(awk -v f="$TARBALL" '$2 == f || $2 == "*"f {print $1}' "${TMP}/SHA256SUMS")
[ -z "$expected" ] && err "no checksum entry for ${TARBALL} in SHA256SUMS"

actual=$(cd "$TMP" && $SHASUM "$TARBALL" | awk '{print $1}')
[ "$expected" != "$actual" ] && err "checksum mismatch (expected $expected, got $actual)"
say "checksum ok"

# Extract and install.
say "extracting"
tar -xzf "${TMP}/${TARBALL}" -C "$TMP"
STAGE="${TMP}/citum-${VER_BARE}-${TARGET}"

mkdir -p "$INSTALL_DIR"
for c in $SELECTED; do
  src="${STAGE}/${c}${EXE_SUFFIX}"
  if [ ! -f "$src" ]; then
    # citum-migrate is not included in musl Linux tarballs because rusty_v8
    # (its embedded V8 dependency) does not publish prebuilt musl static libs.
    if [ "$c" = "citum-migrate" ]; then
      printf '%s\n' "citum-installer: warning: citum-migrate has no prebuilt binary for ${TARGET}."
      printf '%s\n' "  Install from source: cargo install citum-migrate --locked"
      continue
    fi
    err "tarball is missing ${c}${EXE_SUFFIX} — release may pre-date this component"
  fi
  install -m 0755 "$src" "${INSTALL_DIR}/${c}${EXE_SUFFIX}"
  say "installed ${c} -> ${INSTALL_DIR}/${c}${EXE_SUFFIX}"
done

# PATH check. If INSTALL_DIR isn't on PATH, point the user at the fix
# in a way that copy-pastes cleanly into their shell rc file.
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    cat <<EOF

  ${INSTALL_DIR} is not on your PATH. Add this line to your shell
  config (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish, etc.):

      export PATH="${INSTALL_DIR}:\$PATH"

  Then restart your shell, or run:

      export PATH="${INSTALL_DIR}:\$PATH"

EOF
  ;;
esac

# Hint with whichever binary is first in the user's selection.
FIRST="${SELECTED%% *}"
say "done. Run: ${FIRST} --help"
