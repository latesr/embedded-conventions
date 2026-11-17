# Requirements

embedded-conventions holds the originals of the rule files and of `xtask/src/shared/`, and `cargo xtask sync` copies them into each repo that syncs from it.

## Terms

- **Shared files.** `CONVENTIONS.md`, `CONTRIBUTING.md`, the rule files that `CONTRIBUTING.md` indexes, and the files in `xtask/src/shared/`.
- **Copy.** A shared file in another repo, at the same path.

## Crates

| Crate | On crates.io | Contents |
|---|---|---|
| `xtask` | No | The checks, the shared commands, and the sync |

## Constraints

- The crates build on stable Rust. The minimum supported Rust version (MSRV) is 1.91, with edition 2024.

## Testing

Development is test first. `TESTING.md` states how a developer writes and checks tests.

## Licence

Users may choose either the MIT licence or the Apache-2.0 licence. The Cargo `license` field is `MIT OR Apache-2.0`. Contributions are licensed on the same terms.
