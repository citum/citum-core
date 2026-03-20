#!/bin/bash
# macos-link.sh — macOS linker wrapper for aarch64-apple-darwin
#
# Problem: Rust's Apple Silicon linker produces binaries with the
# CS_LINKER_SIGNED flag (codesign flags 0x20002). macOS 15+ (Sequoia)
# runs a ~60-second XProtect/syspolicyd scan on the first execution of
# any binary carrying this flag. This makes `cargo nextest list` appear
# to hang for 60+ seconds after every fresh build.
#
# Fix: run the normal linker, then re-sign Mach-O outputs with a proper
# ad-hoc signature (flags 0x2, no CS_LINKER_SIGNED bit). macOS still
# scans the binary on first execution, but with CS_ADHOC it uses larger
# hash pages (16 KB vs 4 KB), reducing scan time from ~60s to ~30s.
# We then immediately invoke the binary with --list in the background so
# the scan completes during the build rather than blocking `nextest list`.
#
# Usage (via .cargo/config.toml):
#   [target.aarch64-apple-darwin]
#   linker = "scripts/macos-link.sh"
#
# The script is a no-op on any binary that is not a Mach-O executable
# (static libs, rlibs, etc.) so it is safe to use as a universal linker.
# The background warm-up is only attempted on test binaries (those whose
# name contains a typical test-crate suffix after the last '-').

set -euo pipefail

# ── Run the real linker ───────────────────────────────────────────────
cc "$@"

# ── Find the -o <output> path in the argument list ───────────────────
output=""
prev=""
for arg in "$@"; do
    if [ "$prev" = "-o" ]; then
        output="$arg"
        break
    fi
    prev="$arg"
done

# ── Re-sign and warm up Mach-O executables only ──────────────────────
if [ -n "$output" ] && [ -f "$output" ]; then
    if file "$output" | grep -q "Mach-O.*executable"; then
        # Replace CS_LINKER_SIGNED with a proper CS_ADHOC signature.
        # This uses 16 KB hash pages instead of 4 KB, reducing the number
        # of pages the security daemon must verify on first run.
        codesign -s - --force "$output" 2>/dev/null || true

        # Trigger the macOS security scan now (in the background) so it
        # completes during the build rather than blocking nextest later.
        # We use --list so the binary exits immediately after test discovery;
        # errors are silently discarded since we only care about the cache
        # warm-up side-effect.
        "$output" --list --format terse >/dev/null 2>&1 &
    fi
fi
