# ADR-0001: The synced copies of the shared files

**Status: accepted**

Covers: how the shared files, the rule files and `xtask/src/shared/`, and the git hooks reach each repo that syncs from embedded-conventions.

## Context

Several repos follow the same rules and run the same checks. A person who clones one repo must be able to build it and run its checks without a second repo.

## Decision

- embedded-conventions holds the originals. Each repo holds an identical copy, so it builds alone.
- `cargo xtask sync`, run here, is the only way a copy changes. It copies from `HEAD` and prints that commit, which the repo's sync commit names.
- Sync refuses a copy that matches no committed version of its original, so an edit made in a copy is never overwritten. The history of the shared files therefore keeps every version that a repo holds.
- Sync installs the git hooks, so a change to a hook reaches every repo with the next sync.
- Each repo owns every other file, such as its `check.rs`, because its checks differ.

## Rejected

- **A shared crate from a git tag.** A clone of one repo could not build its `xtask` alone. Each rule change would need a tag and a version bump in every repo, and each difference between repos a configuration type.
- **A shared crate on crates.io.** A clone would build alone, but each rule change would need a public and permanent release.
- **The originals in embedded-devkit.** The devkit depends on the libraries, so a library's rules would come from the repo above it.
- **Copies from the template, never synced again.** Each repo would drift from the others.

## Consequences

- A change to a shared file is a commit here, then a sync and a commit in each repo.
- Rewriting the history of the shared files, such as by a squash, makes sync refuse the copies that each repo holds.
- A new crate dependency of the shared code needs an edit in each repo's `xtask/Cargo.toml`.
