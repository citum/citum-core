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
# Options:
#   --components <list> — comma-separated subset of {citum, citum-server,
#                         citum-migrate}, or the alias `all`
#   --help              — show usage
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
#   curl -fsSL .../install.sh | sh -s -- --components all           # everything
#   curl -fsSL .../install.sh | sh -s -- --components citum,citum-migrate

set -eu

REPO="${CITUM_REPO:-citum/citum-core}"
VERSION="${CITUM_VERSION:-latest}"
INSTALL_DIR="${CITUM_INSTALL_DIR:-$HOME/.local/bin}"

# ---------- helpers ---------------------------------------------------

say()    { printf '%s\n'    "citum-installer: $*"; }
err()    { printf '%s\n' >&2 "citum-installer: error: $*"; exit 1; }

usage() {
  cat <<'EOF'
Usage: install.sh [--components <list>]

Install selected Citum binaries. Components are a comma-separated subset of
citum, citum-server, and citum-migrate; use `all` to install every component.

Examples:
  curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | sh
  curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | sh -s -- --components all
  curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | sh -s -- --components citum,citum-migrate

CITUM_COMPONENTS remains available for non-interactive automation. When both
are set, --components takes precedence.
EOF
}

COMPONENTS_ARG=""
COMPONENTS_ARG_SET=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --components)
      [ "$#" -ge 2 ] || err "--components requires a value"
      COMPONENTS_ARG="$2"
      COMPONENTS_ARG_SET=1
      shift 2
      ;;
    --components=*)
      COMPONENTS_ARG="${1#--components=}"
      COMPONENTS_ARG_SET=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    --)
      shift
      [ "$#" -eq 0 ] || err "unexpected positional argument: $1"
      ;;
    *) err "unknown option: $1 (try --help)" ;;
  esac
done

if [ "$COMPONENTS_ARG_SET" -eq 1 ]; then
  COMPONENTS_RAW="$COMPONENTS_ARG"
  COMPONENTS_SOURCE="--components"
else
  COMPONENTS_RAW="${CITUM_COMPONENTS:-citum}"
  COMPONENTS_SOURCE="CITUM_COMPONENTS"
fi

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

# citum-migrate has no musl prebuilt (rusty_v8 doesn't publish musl static
# libs) but does have an x86_64 gnu/glibc one — see scripts/release-binary.sh.
# This maps the x86_64 musl target to that counterpart so install.sh can fetch
# citum-migrate instead of skipping it.
migrate_fallback_target() {
  case "$1" in
    x86_64-unknown-linux-musl)  echo "x86_64-unknown-linux-gnu" ;;
  esac
}

# Downloads, verifies, and extracts the release tarball for target $1 into
# $TMP (uses the SHA256SUMS already fetched into $TMP — one manifest covers
# every target's tarball). Echoes the extracted stage dir path on success.
# On any failure, warns and returns 1 instead of exiting, so callers can
# treat this as an optional fetch. Callers capture this via command
# substitution, so all status output here must go to stderr — stdout is
# reserved for the single path this function echoes on success.
fetch_tarball() {
  t="$1"
  tarball="citum-${VER_BARE}-${t}.tar.gz"

  printf '%s\n' "citum-installer: downloading ${tarball}" >&2
  if ! curl --fail --location --silent --show-error \
    --output "${TMP}/${tarball}" "${BASE_URL}/${tarball}"; then
    printf '%s\n' "citum-installer: warning: failed to download ${tarball}" >&2
    return 1
  fi

  expected=$(awk -v f="$tarball" '$2 == f || $2 == "*"f {print $1}' "${TMP}/SHA256SUMS")
  if [ -z "$expected" ]; then
    printf '%s\n' "citum-installer: warning: no checksum entry for ${tarball} in SHA256SUMS" >&2
    return 1
  fi
  actual=$(cd "$TMP" && $SHASUM "$tarball" | awk '{print $1}')
  if [ "$expected" != "$actual" ]; then
    printf '%s\n' "citum-installer: warning: checksum mismatch for ${tarball} (expected $expected, got $actual)" >&2
    return 1
  fi

  printf '%s\n' "citum-installer: extracting ${tarball}" >&2
  tar -xzf "${TMP}/${tarball}" -C "$TMP"
  echo "${TMP}/citum-${VER_BARE}-${t}"
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
[ -z "$SELECTED" ] && err "${COMPONENTS_SOURCE} is empty (valid: citum, citum-server, citum-migrate, all)"
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

# citum-migrate has no musl prebuilt (see fetch_tarball's doc comment); fetch
# it from the x86_64 gnu counterpart tarball when one is available.
MIGRATE_STAGE=""
case "$TARGET" in
  *-linux-musl)
    case " $SELECTED " in
      *' citum-migrate '*)
        MIGRATE_TARGET="$(migrate_fallback_target "$TARGET")"
        if [ -n "$MIGRATE_TARGET" ]; then
          say "citum-migrate has no musl prebuilt; fetching it from ${MIGRATE_TARGET} instead"
          MIGRATE_STAGE="$(fetch_tarball "$MIGRATE_TARGET" || true)"
        fi
        ;;
    esac
    ;;
esac

mkdir -p "$INSTALL_DIR"
for c in $SELECTED; do
  src="${STAGE}/${c}${EXE_SUFFIX}"
  if [ ! -f "$src" ] && [ "$c" = "citum-migrate" ] && [ -n "$MIGRATE_STAGE" ]; then
    src="${MIGRATE_STAGE}/${c}${EXE_SUFFIX}"
  fi
  if [ ! -f "$src" ]; then
    # On musl Linux, a citum-migrate gnu-fallback fetch failure isn't a
    # packaging regression — fall back to cargo install instead of failing.
    # On all other targets this is a real packaging regression — fail fast.
    case "$TARGET" in
      *-linux-musl)
        if [ "$c" = "citum-migrate" ]; then
          printf '%s\n' "citum-installer: warning: citum-migrate has no prebuilt binary for ${TARGET} (gnu fallback unavailable)."
          printf '%s\n' "  Install from source: cargo install citum-migrate --locked"
          continue
        fi
        ;;
    esac
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
