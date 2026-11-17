# ADR-0001: The synced copies of the shared rules and checks

**Status: accepted**

Covers: how `CONVENTIONS.md`, `CONTRIBUTING.md`, the rule files that `CONTRIBUTING.md` indexes, `xtask/src/shared/`, and the git hooks reach each repo that syncs from embedded-conventions.

## Context

Several repos need the same checks, the same hooks, and the same rules. The rules change most weeks, and a rule that differs between repos is a fault. A person who clones one repo must be able to build and run its checks without a second repo. The checks and the fuzz targets differ between repos. embedded-devkit depends on the libraries. Rules kept there would reach each library from the repo above it.

## Decision

- embedded-conventions holds the originals of `CONVENTIONS.md`, `CONTRIBUTING.md`, the rule files that `CONTRIBUTING.md` indexes, and `xtask/src/shared/`. Each repo, embedded-devkit too, holds an identical copy, so it builds and runs its checks alone.
- `cargo xtask sync` in embedded-conventions is the only way a copy changes. It copies from `HEAD`, so the sync commit in a repo can name the commit that it holds.
- Sync fails on a copy that matches no committed version of its original, and leaves that copy as it is. An edit in place then moves to embedded-conventions and is not lost.
- Sync installs the git hooks that the shared `hooks` module writes, so a change to a hook reaches every repo with the next sync.
- Each repo owns every other file, such as its `xtask/src/main.rs` and `check.rs`, because its checks differ from the other repos' checks.
- Each repo's `main.rs` flattens the shared `Command` enum once, so a repo does not change for a new shared command.
- Code that builds for a board is not shared through sync. It lives in embedded-devkit, which depends on the libraries.

## Rejected

- **A shared crate from a git tag.** A clone of one repo could not build its `xtask` without embedded-conventions. Each rule change would need a tag and a version bump in every repo, and each difference between repos would need a configuration type.
- **A shared crate on crates.io.** A clone would build alone, but each rule change would need a release, and a release is public and permanent. It becomes the better choice for the generic modules once a third chip family exists.
- **The originals in embedded-devkit.** A library's rules would come from the repo that depends on it, and each rule change would sit in a history of board work.
- **Copies from the template, never synced again.** Each repo would drift from the others with each change.

## Consequences

- A change to a shared file is a commit in embedded-conventions, then a sync and a commit in each repo.
- A new crate dependency of the shared modules needs an edit in each repo, because each repo owns its `xtask/Cargo.toml`.
- The template holds none of the synced files. Sync supplies them to a new repo.
