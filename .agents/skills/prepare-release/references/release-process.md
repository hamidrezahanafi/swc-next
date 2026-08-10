# SWC Next release process

## Invariants

- All publishable `swc_next_*` Rust crates, N-API crates, `@swc-next/core`, `@swc-next/parser`, and generated platform packages use one version.
- Release changes arrive through a reviewed `release/v<version>` PR.
- The exact PR title is `chore(release): prepare for v<version>`.
- Version preparation never publishes packages or starts GitHub Actions workflows.
- Both release workflows only publish a commit already merged into `main`.

## Native packages

Each facade owns an independent binding family:

- `@swc-next/core` loads `@swc-next/core-binding-*`.
- `@swc-next/parser` loads `@swc-next/parser-binding-*`.

The supported targets are macOS arm64/x64, Linux glibc arm64/x64, and Windows x64 MSVC. Exact-version `optionalDependencies` are generated during npm release assembly instead of being maintained in source manifests.

## Prepare

Use `scripts/prepare-bump-pr.sh <patch|minor|major>`. It requires a clean, up-to-date `main`, changes the global version, updates lockfiles, regenerates both checked-in N-API loaders, runs no-publish checks, and opens the bump PR.

`bump-pr-check.yml` independently validates the branch, title, version graph, lockfiles, generated loaders, builds, tests, and package tarballs.

## Publish

- `release-crates.yml` manually publishes Rust crates with crates.io trusted publishing.
- `release-npm.yml` manually builds both bindings in each target job, assembles the platform packages, and publishes with npm trusted publishing and provenance.
- npm bindings publish before the two facades. Existing exact versions are skipped so a partially completed npm release can be resumed.
- The workflows are independent and npm publication is not an atomic multi-package transaction.

## First npm release

npm only allows trusted publishing to be configured for package names that already exist. Before the first automated release, seed the two facade names and all ten platform-package names once through an authenticated maintainer process. Then configure `release-npm.yml` with environment `release` as the trusted publisher for every package. The release workflow checks this prerequisite before publishing anything; the skill never performs the bootstrap.
