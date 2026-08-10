---
name: prepare-release
description: Prepare and validate synchronized SWC Next release bump pull requests. Use when asked to bump the repository version, create a release PR, check release-version consistency, or explain the crates.io and npm release process.
---

# Prepare Release

Keep every publishable Rust crate, N-API crate, npm facade, and generated native binding on one global version. Prepare a reviewed PR; never publish locally or dispatch a release workflow.

## Prepare a bump PR

1. Read [references/release-process.md](references/release-process.md).
2. Confirm the requested bump is `patch`, `minor`, or `major`.
3. Require a clean checkout of the latest `main`.
4. Preview the next version:

   ```bash
   .agents/skills/prepare-release/scripts/prepare-bump-pr.sh patch --dry-run
   ```

5. When the user has asked to create the PR, run the same command without `--dry-run`. The script updates all versions, regenerates both N-API loaders, validates the result, creates `release/v<version>`, pushes it, and opens the PR.
6. Report the PR URL and the checks that ran.

Do not hand-edit a subset of version files. Do not run `cargo publish`, `npm publish`, `pnpm publish`, or GitHub workflow dispatch commands.

## Validate a bump

Run the deterministic version validator:

```bash
node .agents/skills/prepare-release/scripts/release-version.mjs check
```

For a release branch, also run:

```bash
PR_TITLE='chore(release): prepare for v0.12.0' \
  .agents/skills/prepare-release/scripts/check-bump-pr.sh --base-ref origin/main
```

The branch must be `release/v<version>` and the PR title must be `chore(release): prepare for v<version>`.

## Release handoff

After review and `bump-pr-check.yml` pass, a human merges the PR. A human then independently starts either `release-crates.yml` or `release-npm.yml` from `main`. Never start these workflows from this skill.
