# Requirements

embedded-conventions holds the originals of the rule files and of `xtask/src/shared/`, and `cargo xtask sync` copies them into each repo that syncs from it.

Each requirement has an ID, such as S-3. Tests cite the ID. An ID is never reused after its requirement is removed. The letter names the requirement's section:

- S, the sync, `cargo xtask sync`

## Terms

- **Shared files.** `CONVENTIONS.md`, `CONTRIBUTING.md`, the rule files that `CONTRIBUTING.md` indexes, and the files in `xtask/src/shared/`.
- **Copy.** A shared file in another repo, at the same path.

## Crates

| Crate | On crates.io | Contents |
|---|---|---|
| `xtask` | No | The checks, the shared commands, and the sync |

## Sync: `cargo xtask sync`

`cargo xtask sync <repo>…` runs in embedded-conventions, and copies into each repo named.

- **S-1** It copies each shared file from the `HEAD` of embedded-conventions, and prints that commit.
- **S-2** It copies nothing while a shared file in embedded-conventions has changes that are not committed, and it fails.
- **S-3** It leaves a copy that matches no committed version of its original, and fails, naming the repo and the file.
- **S-4** It removes a file in a repo's `xtask/src/shared/` that is not in `HEAD` when the file matches a committed version, and reports it as edited in place when it does not.
- **S-5** It installs the git hooks in a repo that has a `.git/hooks` folder, and reports that it did not when the folder is missing.

## Constraints

- The crates build on stable Rust. The minimum supported Rust version (MSRV) is 1.91, with edition 2024.

## Testing

Development is test first. `TESTING.md` states how a developer writes and checks tests. This section states what the test suite must contain.

- Each S requirement has a test that names it in an `@verify` comment.

## Licence

Users may choose either the MIT licence or the Apache-2.0 licence. The Cargo `license` field is `MIT OR Apache-2.0`. Contributions are licensed on the same terms.
