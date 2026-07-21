#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/pre-push-policy-base.sh"

repo_dir="$(mktemp -d)"
trap 'rm -rf "$repo_dir"' EXIT

git -C "$repo_dir" init --quiet
git -C "$repo_dir" config user.name "Test User"
git -C "$repo_dir" config user.email "test@example.com"
git -C "$repo_dir" config commit.gpgsign false

printf 'base\n' > "$repo_dir/history"
git -C "$repo_dir" add history
git -C "$repo_dir" commit --quiet -m 'chore: establish test history' -m 'Create a baseline for hook tests.'
base_sha="$(git -C "$repo_dir" rev-parse HEAD)"

printf 'primary\n' >> "$repo_dir/history"
git -C "$repo_dir" commit --quiet -am 'fix(ci): establish primary history' -m 'Advance the trusted primary branch.'
primary_sha="$(git -C "$repo_dir" rev-parse HEAD)"
git -C "$repo_dir" update-ref refs/remotes/origin/main "$primary_sha"

lagging_base="$(pre_push_policy_base "$repo_dir" "$primary_sha" "$base_sha")"
[[ "$lagging_base" == "refs/remotes/origin/main" ]]

git -C "$repo_dir" checkout --quiet -b divergent "$base_sha"
printf 'divergent\n' >> "$repo_dir/history"
git -C "$repo_dir" commit --quiet -am 'fix(ci): establish divergent history' -m 'Create an independent mirror branch.'
divergent_sha="$(git -C "$repo_dir" rev-parse HEAD)"

divergent_base="$(pre_push_policy_base "$repo_dir" "$primary_sha" "$divergent_sha")"
[[ "$divergent_base" == "$divergent_sha" ]]

new_branch_base="$(pre_push_policy_base "$repo_dir" "$primary_sha" "0000000000000000000000000000000000000000")"
[[ "$new_branch_base" == "$primary_sha" ]]
