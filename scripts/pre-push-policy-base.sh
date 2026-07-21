#!/usr/bin/env bash

pre_push_policy_base() {
  local root_dir="${1:?missing repository root}"
  local local_sha="${2:?missing local SHA}"
  local remote_sha="${3:?missing remote SHA}"
  local primary_main="refs/remotes/origin/main"
  local empty_tree="4b825dc642cb6eb9a060e54bf8d69288fbee4904"

  if [[ "$remote_sha" =~ ^0+$ ]]; then
    git -C "$root_dir" merge-base "$local_sha" "$primary_main" 2>/dev/null \
      || printf '%s\n' "$empty_tree"
  elif git -C "$root_dir" show-ref --verify --quiet "$primary_main" \
    && git -C "$root_dir" merge-base --is-ancestor "$remote_sha" "$primary_main"; then
    printf '%s\n' "$primary_main"
  else
    printf '%s\n' "$remote_sha"
  fi
}
