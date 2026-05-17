# Releasing Citum

This file documents how Citum releases work. Most of the pipeline is
automated by `.github/workflows/release.yml`; the human review points
are marked explicitly.

## What ships, where

| Channel | Format | Audience |
|---|---|---|
| **crates.io** | 12 published crates (see list below) | Rust developers (`cargo install citum`, library users) |
| **GitHub Releases** | Cross-platform tarballs + `SHA256SUMS` + `install.sh` | End-users (`curl ... \| sh`), distro packagers |

Out of scope for now (deliberately): Homebrew tap, npm wrapper, Docker
image. Each can be added later as an additional job that consumes the
GitHub Release artifacts already produced.

### Published crates (12)

| Order | Crate | Notes |
|---|---|---|
| 1 | `citum-edtf` | Leaf — Extended Date/Time Format parser |
| 2 | `citum-resolver-api` | Leaf — style resolution interfaces |
| 3 | `csl-legacy` | Leaf — CSL 1.0 parsers (infrastructure) |
| 4 | `citum-schema-data` | Bibliographic data schema |
| 5 | `citum-schema-style` | Style schema |
| 6 | `citum-schema` | Facade re-exporting `-data` + `-style` |
| 7 | `citum-engine` | Core citation processor |
| 8 | `citum-io` | I/O + format conversion |
| 9 | `citum_store` | Style + reference storage |
| 10 | `citum-migrate` | CSL 1.0 → Citum migration |
| 11 | `citum-server` | JSON-RPC server (binary: `citum-server`) |
| 12 | `citum` | CLI (binary: `citum`) |

Publish order matters because each crate's dependents need it
available on crates.io. `scripts/publish-crates.sh` enforces the
order and is idempotent: re-running after partial failure skips
already-published versions.

## Cut a release

The release is automated end-to-end. Day-to-day, the work is:

1. **Merge conventional commits to `main`.** The `detect` and
   `release-pr` jobs in `release.yml` watch every PR merge and open
   (or update) a `release/next` PR with the inferred version bump.

2. **Review the `release/next` PR.** It bumps `[workspace.package].version`,
   regenerates schemas (if relevant), and updates the changelog via
   `git-cliff`. Land it when you're ready to release.

3. **Watch the tag-push pipeline.** Merging `release/next` triggers
   the `auto-tag` job, which pushes `v<x.y.z>` (and `schema-v<x.y.z>`
   when the schema changed). The tag push fans out to:

   - `build` (~10–15 min, parallel matrix across 5 targets)
   - `release` (~1 min, gated on `build`) — creates the GitHub Release
     with tarballs + `SHA256SUMS` + `install.sh`
   - `publish-crates` (~5 min, gated on `build`) — runs
     `scripts/publish-crates.sh` against `CARGO_REGISTRY_TOKEN`

4. **Verify** (described in [§ Post-release verification](#post-release-verification)).

## What fires on tag push

| Job | Trigger | Time | Notes |
|---|---|---|---|
| `build` | `push: tags: ["v*"]` | ~15 min | Matrix: x86_64-musl, aarch64-musl, x86_64-darwin, aarch64-darwin, x86_64-windows. Uses `cross` for aarch64-musl. |
| `release` | `needs: build` | ~1 min | Aggregates per-target `.sha256` into one `SHA256SUMS`; uploads tarballs + `install.sh` to GitHub Release. |
| `publish-crates` | `needs: build` | ~5 min | Idempotent; safe to re-run from the Actions UI. |

The pre-existing `detect` / `release-pr` / `auto-tag` jobs are gated
on `pull_request` events and don't fire on tag push — there's no
overlap.

## Post-release verification

After the tag-push pipeline completes:

```sh
# Library install (Rust developers):
cargo install citum --locked
cargo install citum-server --locked

# End-user install (everyone):
curl -fsSL https://github.com/citum/citum-core/releases/latest/download/install.sh | sh

# Sanity:
citum --version
citum-server --version
```

The install script verifies the SHA256 of the downloaded tarball
against the release's `SHA256SUMS` before extracting; a checksum
mismatch aborts before any files are written.

## First-time setup (one-time, before the very first release)

1. **crates.io account.** Sign in at https://crates.io via GitHub.

2. **`citum` crate name.** As of writing the `citum` name on crates.io
   is squatted by a 2020 placeholder. Email `help@crates.io` citing
   the `rust-lang/crates.io` policy on unused names to request reclaim.
   If denied, rename `[package].name` in `crates/citum-cli/Cargo.toml`
   to `citum-cli` — the binary stays named `citum` via the `[[bin]]`
   block.

3. **API token.** crates.io → Account → API Tokens → Generate, with
   scopes `publish-new` + `publish-update`. Save the value.

4. **GitHub secret.** Repo Settings → Secrets and variables → Actions
   → New repository secret named `CARGO_REGISTRY_TOKEN` (GitHub
   secret names accept only uppercase letters, digits, and
   underscores — no hyphens).

5. **GitHub team.** Confirm `https://github.com/orgs/citum/teams/cargo-release`
   exists (create if not). After the first publish completes, run
   locally:

   ```sh
   for c in citum-edtf citum-resolver-api csl-legacy \
            citum-schema-data citum-schema-style citum-schema \
            citum-engine citum-io citum_store \
            citum-migrate citum-server citum; do
     cargo owner --add github:citum:cargo-release "$c"
   done
   ```

6. **Rotate the token.** After step 5, generate a fresh crates.io
   token scoped to just those 12 crate names (no `publish-new` —
   they all exist now), replace the GitHub secret, and revoke the
   original wide-scope token.

## Recovery

| Failure | Recovery |
|---|---|
| `publish-crates` partial fail | Re-run the job from the Actions UI. The script's idempotency check skips already-published versions. |
| Bad release tarball | `gh release delete v<x.y.z> --cleanup-tag`, fix the issue, re-tag and let the workflow re-run. |
| Bad crate published | `cargo yank --version <x.y.z> <crate>`. **Cannot un-yank with the same version**; bump to the next version and re-publish. |
| Tag pushed before workflow updated | Delete the tag locally (`git tag -d v<x.y.z>`) and on remote (`git push origin :refs/tags/v<x.y.z>`); fix workflow; re-push. |

## Locally dry-run a publish

```sh
./scripts/publish-crates.sh --dry-run
```

Does no uploads. The leaf crates (`citum-edtf`, `citum-resolver-api`,
`csl-legacy`) will succeed cleanly. Dependents fail with
"no matching package … found" because their internal deps aren't on
crates.io yet — that's expected in dry-run mode, and resolves in
production where each successful publish makes the next dependent's
dep available.

## Locally dry-run a binary build

```sh
./scripts/release-binary.sh x86_64-apple-darwin v0.51.0
ls release-out/x86_64-apple-darwin/
```

Produces the tarball + `.sha256` exactly as the CI matrix would. Useful
when changing build flags or adding new bundled files.

## Locally test the installer

```sh
# Against a release that exists:
CITUM_INSTALL_DIR=/tmp/citum-test ./scripts/install.sh
/tmp/citum-test/citum --version

# Against a specific release:
CITUM_VERSION=v0.51.0 CITUM_INSTALL_DIR=/tmp/citum-test ./scripts/install.sh
```
