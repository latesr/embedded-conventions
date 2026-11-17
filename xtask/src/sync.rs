//! Copies the rule files and the shared xtask folder into other repos, and installs the git hooks there.
//!
//! The copy comes from this repo's `HEAD`, so the commit in the other repo can name the commit it holds. A file in
//! the other repo that matches no committed version of this repo's file was edited in place. Sync reports it and
//! leaves it alone, so the edit can move here first. `--overwrite` replaces it instead, once every change to the
//! other repo's copies is committed, so the replaced edit stays in that repo's history.
//!
//! The hooks come from the shared `hooks` module that this binary is built from. Sync refuses to run while that
//! module has changes that are not committed, so the hooks match the copied files.

use crate::shared::{files::RULE_FILES, git, hooks};
use std::fs;
use std::path::{Path, PathBuf};

/// The folder that every repo holds a copy of, relative to each repo's root.
const SHARED: &str = "xtask/src/shared";

/// Every path that sync copies. A file here that this repo deletes stays in the other repos, because only
/// [`SHARED`] is scanned for files to remove.
fn synced() -> Vec<&'static str> {
    RULE_FILES.iter().copied().chain([SHARED]).collect()
}

/// What one sync changed in one repo, and what it left alone. Each entry is a path relative to the repo's root.
#[derive(Debug, Default, PartialEq)]
struct Report {
    added: Vec<String>,
    updated: Vec<String>,
    removed: Vec<String>,
    edited_in_place: Vec<String>,
    /// The copies that matched no committed version, replaced because the sync ran with `--overwrite`.
    overwritten: Vec<String>,
    /// False for a folder that is not a git repository yet.
    hooks_installed: bool,
}

impl Report {
    /// The lines that tell the user what changed. The copies edited in place are reported apart, as the failure.
    fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for (what, paths) in [
            ("added", &self.added),
            ("updated", &self.updated),
            ("removed", &self.removed),
            ("overwrote", &self.overwritten),
        ] {
            lines.extend(paths.iter().map(|path| format!("{what} {path}")));
        }
        lines.push(if self.hooks_installed {
            "installed the git hooks".to_owned()
        } else {
            "no .git/hooks folder, so the git hooks were not installed".to_owned()
        });
        lines
    }
}

/// Syncs each repo in `repos` from embedded-conventions at `source`. Fails when the shared files in `source` have
/// changes that are not committed, or when a repo holds a copy that was edited in place. With `overwrite`, it
/// replaces such a copy instead, and first fails, writing nothing, when a repo is not a git repository or has
/// changes to its copies that are not committed.
pub fn run(source: &Path, repos: &[PathBuf], overwrite: bool) -> Result<(), String> {
    let status = [&["status", "--porcelain", "--"][..], &synced()].concat();
    if !git(source, &status)?.is_empty() {
        return Err(
            "The rule files or xtask/src/shared in embedded-conventions have changes that are not committed. Commit \
             them, then sync."
                .to_owned(),
        );
    }
    if overwrite {
        for repo in repos {
            copies_committed(repo)?;
        }
    }
    let head = String::from_utf8_lossy(&git(source, &["rev-parse", "--short", "HEAD"])?)
        .trim()
        .to_owned();
    let mut edited = Vec::new();
    for repo in repos {
        println!("==> {}", repo.display());
        let report = sync_one(source, repo, overwrite)?;
        for line in report.lines() {
            println!("{line}");
        }
        for path in report.edited_in_place {
            edited.push(format!("{}: {path}", repo.display()));
        }
    }
    println!("{}", synced_from(&head));
    if edited.is_empty() {
        return Ok(());
    }
    Err(format!(
        "These copies match no committed version in embedded-conventions, so sync left them alone. Move each edit \
         to embedded-conventions, commit it, and sync again. To discard the edits, commit or discard every change \
         to the copies, then sync with --overwrite.\n{}",
        edited.join("\n")
    ))
}

/// The last line that sync prints, which names the commit that each repo's sync commit holds.
fn synced_from(head: &str) -> String {
    format!("synced from embedded-conventions {head}")
}

/// Fails when `repo` is not the top of its own git repository, or when its copies have changes that are not
/// committed. Then every copy that `--overwrite` replaces stays in the repo's history. A file that the repo's
/// `.gitignore` ignores does not show as a change, so it is replaced without a copy in the history.
fn copies_committed(repo: &Path) -> Result<(), String> {
    let not_a_repo = |detail: String| {
        format!(
            "{}: --overwrite needs a git repository, so each copy it replaces stays in the history. {detail}",
            repo.display()
        )
    };
    // Git searches the parent folders for a repository, so a plain folder inside another repo still answers.
    let top = git(repo, &["rev-parse", "--show-toplevel"]).map_err(not_a_repo)?;
    let top = PathBuf::from(String::from_utf8_lossy(&top).trim());
    let canonical = |path: &Path| fs::canonicalize(path).ok();
    if canonical(&top).is_none() || canonical(&top) != canonical(repo) {
        return Err(not_a_repo(format!("It is inside {}.", top.display())));
    }
    let status = [&["status", "--porcelain", "--"][..], &synced()].concat();
    let changes = git(repo, &status).map_err(not_a_repo)?;
    if changes.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{}: the copies have changes that are not committed. Commit or discard them, then sync with --overwrite.",
        repo.display()
    ))
}

fn sync_one(source: &Path, repo: &Path, overwrite: bool) -> Result<Report, String> {
    let mut report = Report::default();
    let ls_tree = [
        &["ls-tree", "-r", "--name-only", "HEAD", "--"][..],
        &synced(),
    ]
    .concat();
    let listing = git(source, &ls_tree)?;
    let current: Vec<String> = String::from_utf8_lossy(&listing)
        .lines()
        .map(str::to_owned)
        .collect();
    for path in &current {
        let content = git(source, &["show", &format!("HEAD:{path}")])?;
        let target = repo.join(path);
        match fs::read(&target) {
            Ok(existing) if existing == content => continue,
            Ok(existing) if !history(source, path)?.contains(&existing) => {
                if !overwrite {
                    report.edited_in_place.push(path.clone());
                    continue;
                }
                report.overwritten.push(path.clone());
            }
            Ok(_) => report.updated.push(path.clone()),
            Err(_) => report.added.push(path.clone()),
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        fs::write(&target, content).map_err(|error| format!("{}: {error}", target.display()))?;
    }
    let entries = fs::read_dir(repo.join(SHARED))
        .into_iter()
        .flatten()
        .flatten();
    for entry in entries {
        let path = format!("{SHARED}/{}", entry.file_name().to_string_lossy());
        if current.contains(&path) || !entry.path().is_file() {
            continue;
        }
        let existing = fs::read(entry.path()).map_err(|error| format!("{path}: {error}"))?;
        if overwrite || history(source, &path)?.contains(&existing) {
            fs::remove_file(entry.path()).map_err(|error| format!("{path}: {error}"))?;
            report.removed.push(path);
        } else {
            report.edited_in_place.push(path);
        }
    }
    report.removed.sort();
    report.edited_in_place.sort();
    if repo.join(".git").join("hooks").is_dir() {
        hooks::install(repo)?;
        report.hooks_installed = true;
    }
    Ok(report)
}

/// Every committed version of `path` in `source`. The commit that deleted the file is skipped, because it holds no
/// version.
fn history(source: &Path, path: &str) -> Result<Vec<Vec<u8>>, String> {
    let commits = git(source, &["log", "--format=%H", "--", path])?;
    Ok(String::from_utf8_lossy(&commits)
        .lines()
        .filter_map(|commit| git(source, &["show", &format!("{commit}:{path}")]).ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::files::testing::TempRepo;

    /// A source repo with two commits. `a.rs` changed between them, `old.rs` was removed and `new.rs` was added.
    fn source(name: &str) -> TempRepo {
        let source = TempRepo::new(&format!("{name}-source"));
        fs::create_dir_all(source.0.join(SHARED)).unwrap();
        for args in [
            &["init", "--quiet"][..],
            &["config", "user.email", "test@example.com"],
            &["config", "user.name", "test"],
        ] {
            git(&source.0, args).unwrap();
        }
        source.write("xtask/src/shared/a.rs", "a1\n");
        source.write("xtask/src/shared/old.rs", "old\n");
        git(&source.0, &["add", "--all"]).unwrap();
        git(&source.0, &["commit", "--quiet", "-m", "one"]).unwrap();
        source.write("xtask/src/shared/a.rs", "a2\n");
        source.write("xtask/src/shared/new.rs", "new\n");
        fs::remove_file(source.0.join("xtask/src/shared/old.rs")).unwrap();
        git(&source.0, &["add", "--all"]).unwrap();
        git(&source.0, &["commit", "--quiet", "-m", "two"]).unwrap();
        source
    }

    fn repo(name: &str) -> TempRepo {
        let repo = TempRepo::new(name);
        fs::create_dir_all(repo.0.join(SHARED)).unwrap();
        repo
    }

    fn read(repo: &TempRepo, path: &str) -> Option<String> {
        fs::read_to_string(repo.0.join(path)).ok()
    }

    fn paths(names: &[&str]) -> Vec<String> {
        names
            .iter()
            .map(|name| format!("{SHARED}/{name}"))
            .collect()
    }

    // @verify S-1, S-4
    #[test]
    fn an_old_copy_is_brought_up_to_date() {
        let source = source("sync-old-copy");
        let repo = repo("sync-old-copy");
        repo.write("xtask/src/shared/a.rs", "a1\n");
        repo.write("xtask/src/shared/old.rs", "old\n");

        let report = sync_one(&source.0, &repo.0, false).unwrap();
        assert_eq!(
            report,
            Report {
                added: paths(&["new.rs"]),
                updated: paths(&["a.rs"]),
                removed: paths(&["old.rs"]),
                edited_in_place: Vec::new(),
                overwritten: Vec::new(),
                hooks_installed: false,
            }
        );
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a2\n")
        );
        assert_eq!(
            read(&repo, "xtask/src/shared/new.rs").as_deref(),
            Some("new\n")
        );
        assert_eq!(read(&repo, "xtask/src/shared/old.rs"), None);
    }

    #[test]
    fn a_current_copy_is_left_alone_and_reported_as_nothing() {
        let source = source("sync-current");
        let repo = repo("sync-current");
        repo.write("xtask/src/shared/a.rs", "a2\n");
        repo.write("xtask/src/shared/new.rs", "new\n");
        assert_eq!(
            sync_one(&source.0, &repo.0, false).unwrap(),
            Report::default()
        );
    }

    // @verify S-3, S-4
    #[test]
    fn a_copy_edited_in_place_is_reported_and_kept() {
        let source = source("sync-edited");
        let repo = repo("sync-edited");
        repo.write("xtask/src/shared/a.rs", "a1 edited\n");
        repo.write("xtask/src/shared/mine.rs", "mine\n");

        let report = sync_one(&source.0, &repo.0, false).unwrap();
        assert_eq!(report.edited_in_place, paths(&["a.rs", "mine.rs"]));
        assert_eq!(report.added, paths(&["new.rs"]));
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a1 edited\n")
        );
        assert_eq!(
            read(&repo, "xtask/src/shared/mine.rs").as_deref(),
            Some("mine\n")
        );
    }

    // @verify S-1, S-5
    #[test]
    fn a_repo_without_the_folder_gets_every_file() {
        let source = source("sync-empty");
        let repo = TempRepo::new("sync-empty");
        let report = sync_one(&source.0, &repo.0, false).unwrap();
        assert_eq!(report.added, paths(&["a.rs", "new.rs"]));
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a2\n")
        );
        assert!(!report.hooks_installed);
    }

    #[test]
    fn the_report_names_each_change_and_a_missing_hooks_folder() {
        let report = Report {
            added: vec!["a".to_owned()],
            updated: vec!["u".to_owned()],
            removed: vec!["r".to_owned()],
            edited_in_place: vec!["e".to_owned()],
            overwritten: Vec::new(),
            hooks_installed: false,
        };
        assert_eq!(
            report.lines(),
            [
                "added a",
                "updated u",
                "removed r",
                "no .git/hooks folder, so the git hooks were not installed",
            ]
        );
        let installed = Report {
            hooks_installed: true,
            ..Report::default()
        };
        assert_eq!(installed.lines(), ["installed the git hooks"]);
    }

    // @verify S-5
    #[test]
    fn the_git_hooks_are_installed_in_a_git_repository() {
        let source = source("sync-hooks");
        let repo = repo("sync-hooks");
        fs::create_dir_all(repo.0.join(".git/hooks")).unwrap();
        repo.write(".git/hooks/commit-msg", "old\n");

        let report = sync_one(&source.0, &repo.0, false).unwrap();
        assert!(report.hooks_installed);
        let commit_msg = read(&repo, ".git/hooks/commit-msg").unwrap();
        assert!(
            commit_msg.contains("cargo xtask links --message"),
            "{commit_msg}"
        );
        let pre_commit = read(&repo, ".git/hooks/pre-commit").unwrap();
        assert!(pre_commit.contains("cargo xtask check"), "{pre_commit}");
    }

    // @verify S-2, S-3
    #[test]
    fn run_fails_on_uncommitted_source_changes_and_on_an_edited_copy() {
        let source = source("sync-run");
        let repo = repo("sync-run");
        let repos = [repo.0.clone()];
        repo.write("xtask/src/shared/a.rs", "a1 edited\n");
        let error = run(&source.0, &repos, false).unwrap_err();
        assert!(
            error.contains(&format!("{}: xtask/src/shared/a.rs", repo.0.display())),
            "{error}"
        );
        assert!(error.contains("to embedded-conventions"), "{error}");
        assert!(error.contains("--overwrite"), "{error}");
        assert_eq!(
            read(&repo, "xtask/src/shared/new.rs").as_deref(),
            Some("new\n")
        );

        repo.write("xtask/src/shared/a.rs", "a2\n");
        assert_eq!(run(&source.0, &repos, false), Ok(()));

        source.write("xtask/src/shared/a.rs", "a3\n");
        let error = run(&source.0, &repos, false).unwrap_err();
        assert!(
            error.contains("in embedded-conventions have changes that are not committed"),
            "{error}"
        );
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a2\n")
        );
    }

    // @verify S-1
    #[test]
    fn the_last_line_names_embedded_conventions_and_its_commit() {
        assert_eq!(
            synced_from("1a2b3c4"),
            "synced from embedded-conventions 1a2b3c4"
        );
    }

    // @verify S-1, S-2, S-3
    #[test]
    fn the_rule_files_are_synced_to_the_repo_root() {
        let source = source("sync-conventions");
        let rule_files = [
            "CHECKS.md",
            "CODE.md",
            "CONTRIBUTING.md",
            "CONVENTIONS.md",
            "DOCS.md",
            "EMBEDDED.md",
            "TESTING.md",
            "WRITING.md",
        ];
        for file in rule_files {
            source.write(file, &format!("{file}\n"));
        }
        source.write("README.md", "the source's own\n");
        git(&source.0, &["add", "--all"]).unwrap();
        git(&source.0, &["commit", "--quiet", "-m", "three"]).unwrap();
        let repo = repo("sync-conventions");
        let repos = [repo.0.clone()];

        let report = sync_one(&source.0, &repo.0, false).unwrap();
        let mut expected: Vec<String> = rule_files.iter().map(|file| (*file).to_owned()).collect();
        expected.extend(paths(&["a.rs", "new.rs"]));
        assert_eq!(report.added, expected);
        assert_eq!(read(&repo, "README.md"), None);
        assert_eq!(
            read(&repo, "CONVENTIONS.md").as_deref(),
            Some("CONVENTIONS.md\n")
        );
        assert_eq!(read(&repo, "WRITING.md").as_deref(), Some("WRITING.md\n"));

        repo.write("CONVENTIONS.md", "rules edited\n");
        let report = sync_one(&source.0, &repo.0, false).unwrap();
        assert_eq!(report.edited_in_place, vec!["CONVENTIONS.md".to_owned()]);

        repo.write("CONVENTIONS.md", "CONVENTIONS.md\n");
        source.write("CONVENTIONS.md", "rules changed\n");
        let error = run(&source.0, &repos, false).unwrap_err();
        assert!(error.contains("not committed"), "{error}");
    }

    /// `repo` as a git repository, with everything in it committed.
    fn commit_all(repo: &TempRepo) {
        for args in [
            &["init", "--quiet"][..],
            &["config", "user.email", "test@example.com"],
            &["config", "user.name", "test"],
            &["add", "--all"],
            &["commit", "--quiet", "-m", "start"],
        ] {
            git(&repo.0, args).unwrap();
        }
    }

    // @verify S-6
    #[test]
    fn overwrite_replaces_an_edited_copy_and_removes_an_unknown_shared_file() {
        let source = source("sync-overwrite");
        let repo = repo("sync-overwrite");
        repo.write("xtask/src/shared/a.rs", "a1 edited\n");
        repo.write("xtask/src/shared/mine.rs", "mine\n");

        let report = sync_one(&source.0, &repo.0, true).unwrap();
        assert_eq!(
            report,
            Report {
                added: paths(&["new.rs"]),
                removed: paths(&["mine.rs"]),
                overwritten: paths(&["a.rs"]),
                ..Report::default()
            }
        );
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a2\n")
        );
        assert_eq!(read(&repo, "xtask/src/shared/mine.rs"), None);
    }

    // @verify S-6
    #[test]
    fn overwrite_runs_only_when_every_change_to_the_copies_is_committed() {
        let source = source("sync-overwrite-guard");
        let repo = repo("sync-overwrite-guard");
        repo.write("xtask/src/shared/a.rs", "a1 edited\n");
        commit_all(&repo);
        let repos = [repo.0.clone()];

        assert_eq!(run(&source.0, &repos, true), Ok(()));
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a2\n")
        );

        repo.write("xtask/src/shared/a.rs", "a2 edited\n");
        let clean = TempRepo::new("sync-overwrite-guard-clean");
        fs::create_dir_all(clean.0.join(SHARED)).unwrap();
        clean.write("xtask/src/shared/a.rs", "a1 edited\n");
        commit_all(&clean);
        let error = run(&source.0, &[clean.0.clone(), repo.0.clone()], true).unwrap_err();
        assert!(error.contains("not committed"), "{error}");
        assert!(error.contains(&repo.0.display().to_string()), "{error}");
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a2 edited\n")
        );
        assert_eq!(
            read(&clean, "xtask/src/shared/a.rs").as_deref(),
            Some("a1 edited\n")
        );
    }

    // @verify S-6
    #[test]
    fn overwrite_refuses_a_folder_that_is_not_a_git_repository() {
        let source = source("sync-overwrite-no-git");
        let repo = repo("sync-overwrite-no-git");
        repo.write("xtask/src/shared/a.rs", "a1 edited\n");
        let repos = [repo.0.clone()];
        let error = run(&source.0, &repos, true).unwrap_err();
        assert!(error.contains("git repository"), "{error}");
        assert_eq!(
            read(&repo, "xtask/src/shared/a.rs").as_deref(),
            Some("a1 edited\n")
        );
    }

    // @verify S-6
    #[test]
    fn overwrite_refuses_a_folder_inside_another_repository() {
        let source = source("sync-overwrite-nested");
        let outer = TempRepo::new("sync-overwrite-nested");
        outer.write(".gitignore", "inner/\n");
        commit_all(&outer);
        fs::create_dir_all(outer.0.join("inner").join(SHARED)).unwrap();
        outer.write("inner/xtask/src/shared/a.rs", "a1 edited\n");
        let repos = [outer.0.join("inner")];
        let error = run(&source.0, &repos, true).unwrap_err();
        assert!(error.contains("git repository"), "{error}");
        assert_eq!(
            read(&outer, "inner/xtask/src/shared/a.rs").as_deref(),
            Some("a1 edited\n")
        );
    }

    #[test]
    fn the_report_names_each_overwritten_copy() {
        let report = Report {
            overwritten: vec!["o".to_owned()],
            hooks_installed: true,
            ..Report::default()
        };
        assert_eq!(report.lines(), ["overwrote o", "installed the git hooks"]);
    }
}
