#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../../.." && pwd)"

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <patch|minor|major> [--dry-run]" >&2
  exit 2
fi

bump="$1"
dry_run=false
if [[ ${2:-} == "--dry-run" ]]; then
  dry_run=true
elif [[ $# -eq 2 ]]; then
  echo "Unknown argument: $2" >&2
  exit 2
fi

case "$bump" in
  patch|minor|major) ;;
  *)
    echo "Bump must be patch, minor, or major." >&2
    exit 2
    ;;
esac

cd "$REPO_ROOT"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Release preparation requires a clean worktree." >&2
  exit 1
fi

current_version="$(node "$SCRIPT_DIR/release-version.mjs" check)"
next_version="$(node "$SCRIPT_DIR/release-version.mjs" bump "$bump")"

if [[ "$(git branch --show-current)" != "main" ]]; then
  echo "Run release preparation from main." >&2
  exit 1
fi

if $dry_run; then
  if ! git rev-parse --verify origin/main >/dev/null 2>&1; then
    echo "Dry-run requires an origin/main reference." >&2
    exit 1
  fi
  if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
    echo "Local main must match origin/main." >&2
    exit 1
  fi
  echo "Would prepare v${next_version} from v${current_version}."
  echo "Would create release/v${next_version} with title: chore(release): prepare for v${next_version}"
  exit 0
fi

for command in git node pnpm cargo git-cliff gh; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Required command not found: $command" >&2
    exit 1
  fi
done

git fetch origin main
if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
  echo "Local main must match origin/main." >&2
  exit 1
fi

branch="release/v${next_version}"
title="chore(release): prepare for v${next_version}"
git switch -c "$branch"

node "$SCRIPT_DIR/release-version.mjs" write "$next_version"
pnpm install --lockfile-only
cargo check --workspace
pnpm build:napi
pnpm --filter @swc-next/parser test
node "$SCRIPT_DIR/release-version.mjs" check "$next_version"
PR_TITLE="$title" "$SCRIPT_DIR/check-bump-pr.sh" --base-ref main

body_file="$(mktemp)"
trap 'rm -f "$body_file"' EXIT
git-cliff --tag "v${next_version}" --unreleased --output "$body_file"

git add --all
git commit -m "$title"
git push --set-upstream origin "$branch"
gh pr create --base main --head "$branch" --title "$title" --body-file "$body_file"
