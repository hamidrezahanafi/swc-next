#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../../.." && pwd)"
BASE_REF="origin/main"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-ref)
      BASE_REF="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

cd "$REPO_ROOT"

version="$(node "$SCRIPT_DIR/release-version.mjs" check)"
head_ref="${GITHUB_HEAD_REF:-$(git branch --show-current)}"
expected_branch="release/v${version}"
expected_title="chore(release): prepare for v${version}"

if [[ "$head_ref" != "$expected_branch" ]]; then
  echo "Release branch must be $expected_branch, got $head_ref" >&2
  exit 1
fi

if [[ "${PR_TITLE:-}" != "$expected_title" ]]; then
  echo "Release PR title must be '$expected_title', got '${PR_TITLE:-<unset>}'" >&2
  exit 1
fi

base_version="$(git show "$BASE_REF:napi/core/package.json" | node -e '
let source = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", chunk => source += chunk);
process.stdin.on("end", () => process.stdout.write(JSON.parse(source).version));
')"
node "$SCRIPT_DIR/release-version.mjs" compare "$base_version" "$version"

while IFS= read -r changed_file; do
  case "$changed_file" in
    Cargo.toml|Cargo.lock|package.json|pnpm-lock.yaml|crates/*/Cargo.toml|napi/*/Cargo.toml|napi/core/package.json|napi/parser/package.json|napi/core/src-js/binding.js|napi/parser/src-js/binding.js|napi/core/src-js/binding.d.ts|napi/parser/src-js/binding.d.ts)
      ;;
    *)
      echo "Unexpected file in release PR: $changed_file" >&2
      exit 1
      ;;
  esac
done < <(git diff --name-only "$BASE_REF")

echo "Validated release PR for v${version}."
