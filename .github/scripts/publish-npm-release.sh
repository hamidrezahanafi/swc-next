#!/usr/bin/env bash
set -euo pipefail

if [[ "${GITHUB_ACTIONS:-}" != "true" || "${GITHUB_REF:-}" != "refs/heads/main" ]]; then
  echo "npm release publishing is only allowed in GitHub Actions on main." >&2
  exit 1
fi

if [[ $# -ne 4 ]]; then
  echo "Usage: $0 <core-bindings> <parser-bindings> <core-package> <parser-package>" >&2
  exit 2
fi

missing_packages=()

check_package_exists() {
  local package_dir="$1"
  local package_name
  package_name="$(node -p "require('./${package_dir}/package.json').name")"
  if ! npm view "$package_name" name --json >/dev/null 2>&1; then
    missing_packages+=("$package_name")
  fi
}

check_binding_tree() {
  local root="$1"
  while IFS= read -r package_json; do
    check_package_exists "$(dirname "$package_json")"
  done < <(find "$root" -type f -name package.json | sort)
}

check_binding_tree "$1"
check_binding_tree "$2"
check_package_exists "$3"
check_package_exists "$4"

if [[ ${#missing_packages[@]} -gt 0 ]]; then
  echo "Trusted publishing requires every package to exist before this workflow can publish it." >&2
  printf 'Missing npm package: %s\n' "${missing_packages[@]}" >&2
  echo "Seed the package names once, then configure release-npm.yml as their trusted publisher." >&2
  exit 1
fi

publish_if_needed() {
  local package_dir="$1"
  local package_name
  local package_version
  package_name="$(node -p "require('./${package_dir}/package.json').name")"
  package_version="$(node -p "require('./${package_dir}/package.json').version")"

  if npm view "${package_name}@${package_version}" version --json >/dev/null 2>&1; then
    echo "Skipping existing ${package_name}@${package_version}."
    return
  fi

  npm publish "$package_dir" --access public --provenance
}

publish_binding_tree() {
  local root="$1"
  while IFS= read -r package_json; do
    publish_if_needed "$(dirname "$package_json")"
  done < <(find "$root" -type f -name package.json | sort)
}

publish_binding_tree "$1"
publish_binding_tree "$2"
publish_if_needed "$3"
publish_if_needed "$4"
