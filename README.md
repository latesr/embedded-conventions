# embedded-conventions

embedded-conventions holds the rules, the shared checks, and the git hooks of the embedded repos, and the template for a new one. Each repo holds a copy of the rule files and of `xtask/src/shared/`, so it builds and runs its checks alone. `cargo xtask sync` here keeps the copies current.

- [`CONTRIBUTING.md`](CONTRIBUTING.md) states what every change must meet, and indexes the rule files.
- [`CONVENTIONS.md`](CONVENTIONS.md) states how the user and the user's agents work.
- [`AGENTS.md`](AGENTS.md) states how to change a shared file, sync it, and start a new repo from `template/`.
- [`REQUIREMENTS.md`](REQUIREMENTS.md) states what sync guarantees. [`docs/`](docs/README.md) holds the Architectural Decision Records (ADRs).

## Licence

Users may choose either the [MIT licence](LICENSE-MIT) or the [Apache-2.0 licence](LICENSE-APACHE).
