@CONVENTIONS.md
@CONTRIBUTING.md
@CODE.md
@TESTING.md

# embedded-conventions

embedded-conventions holds the rules and the shared checks of the repos that sync from it, and the template for a new repo. Each repo takes copies of the shared files through `cargo xtask sync`.

## Shared files

`CONVENTIONS.md`, `CONTRIBUTING.md`, the rule files that `CONTRIBUTING.md` indexes, `xtask/src/shared/`, and the git hooks are the originals. Each repo that syncs from here holds a copy.

1. Edit the originals here, and commit.
2. Run `cargo xtask sync ../<repo>`. Sync copies from `HEAD`, refuses while the shared files have changes that are not committed, and installs the hooks from the shared `hooks` module.
3. Commit the copy in each repo. The subject names the commit, such as "Sync the shared files from embedded-conventions 1a2b3c4".

## The template

`template/` is a cargo-generate template for a new repo. It is not built as part of this workspace. To start a repo beside the others, in the folder that holds this repo:

```
cargo generate --path embedded-conventions/template --name <repo> --define "description=<one sentence>"
git -C <repo> branch -m main
cd embedded-conventions
cargo xtask sync ../<repo>
```

Sync supplies the rule files, `xtask/src/shared/`, and the git hooks, so the template does not hold them. cargo-generate runs `git init` in the new repo, so the hooks have a folder to go in. It names the branch `master`, and the CI workflow builds `main`. After generation the new repo owns every other file.

Before committing a change to the template, generate a repo from it in a scratch folder, sync it, and run `cargo xtask check` there.
