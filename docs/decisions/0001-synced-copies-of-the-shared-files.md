# ADR-0001: The synced copies of the shared files

**Status: accepted**

Covers: how the shared files (the rule files and `xtask/src/shared/`) and the git hooks reach each synced repo, a repo that holds copies of them.

## Context

Several repos follow the same rules and run the same checks. A person who clones one repo must be able to build it and run its checks without a second repo.

## Decision

- embedded-conventions holds the originals. Each synced repo holds an identical copy, and does not depend on embedded-conventions to build.
- `cargo xtask sync`, run in embedded-conventions, is the only way a copy changes. It copies from `HEAD` and prints that commit, which the synced repo's sync commit names.
- Sync refuses to overwrite a copy that matches no committed version in embedded-conventions, so an edit made in a synced repo is not lost. `--overwrite` replaces such a copy, and refuses while the synced repo has uncommitted changes to its copies.
- Sync installs the git hooks, so a change to a hook reaches every synced repo with the next sync.

## Rejected

- **A shared crate from a git tag.** A clone of one repo could not build its `xtask` alone. Each rule change would need a tag and a version bump in every repo, and each difference between repos would need a configuration type.
- **A shared crate on crates.io.** A clone would build alone, but each rule change would need a public and permanent release.
- **The originals in embedded-devkit.** The devkit depends on the libraries, so a library's rules would come from the repo above it.
- **Copies from the template, never synced again.** Each repo would drift from the others.

## Consequences

- A change to a shared file is a commit in embedded-conventions, then a sync and a commit in each synced repo.
- A squash or other rewrite of the history in embedded-conventions deletes the old versions. Sync then treats a copy that is only out of date as edited, and `--overwrite` is needed in each synced repo.
- A new crate dependency of the shared code needs an edit in each synced repo's `xtask/Cargo.toml`.
