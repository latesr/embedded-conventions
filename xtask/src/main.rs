//! Repository tasks. `cargo xtask <command>` runs one.

mod check;
mod shared;
mod sync;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(about = "Repository tasks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the checks with every feature on and then off.
    Check {
        /// Run every check, over each feature alone and each pair, as CI does.
        #[arg(long)]
        full: bool,
    },
    #[command(flatten)]
    Shared(shared::Command),
    /// Copy the rule files and xtask/src/shared into each repo, install its git hooks, and report each copy that was
    /// edited in place.
    Sync {
        /// The folders of the repos to copy into.
        #[arg(required = true)]
        repos: Vec<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let root = repo_root();
    let result = match cli.command {
        Command::Check { full } => check::run(&root, full),
        Command::Shared(command) => shared::run(&root, command),
        Command::Sync { repos } => sync::run(&root, &repos),
    };
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

/// The workspace root, which is the parent of the `xtask` crate.
fn repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.to_path_buf())
}
