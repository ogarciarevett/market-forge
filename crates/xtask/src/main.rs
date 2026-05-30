//! xtask — market-forge's dev task runner, pure Rust (no Node/Bun toolchain).
//!
//! It is the single source of truth for regenerating the per-tool agent docs from
//! the hand-edited sources under `.ai/`, plus project lints.
//!
//! Subcommands:
//!   sync-ai [--all-inline]  Regenerate every per-tool agent artifact from `.ai/`.
//!   check-sync              Run `sync-ai`, then `git diff --exit-code` (fails on drift).
//!   lint-catalog            Lint `docs/catalog/**/*.md` against `docs/standards.md` §2.

mod catalog;
mod gitcmd;
mod sync;
mod transforms;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (subcommand, rest) = match args.split_first() {
        Some((first, rest)) => (first.as_str(), rest),
        None => ("sync-ai", &[][..]),
    };

    let result = match subcommand {
        "sync-ai" => cmd_sync_ai(rest),
        "check-sync" => cmd_check_sync(rest),
        "lint-catalog" => cmd_lint_catalog(rest),
        "-h" | "--help" | "help" => {
            print_help();
            return ExitCode::SUCCESS;
        }
        other => {
            eprintln!("xtask: unknown subcommand `{other}`\n");
            print_help();
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("xtask: {err}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    eprintln!(
        "xtask — market-forge dev task runner (pure Rust)\n\n\
         USAGE:\n\
         \x20 cargo xtask sync-ai [--all-inline]   Regenerate per-tool agent configs from .ai/\n\
         \x20 cargo xtask check-sync               Sync then `git diff --exit-code`\n\
         \x20 cargo xtask lint-catalog             Lint docs/catalog entries (standards.md §2)\n"
    );
}

fn cmd_sync_ai(rest: &[String]) -> Result<ExitCode, String> {
    let all_inline = rest.iter().any(|a| a == "--all-inline");
    let root = repo_root()?;
    let summary = sync::run(&root, all_inline).map_err(|e| e.to_string())?;
    summary.print();
    Ok(ExitCode::SUCCESS)
}

fn cmd_check_sync(rest: &[String]) -> Result<ExitCode, String> {
    let all_inline = rest.iter().any(|a| a == "--all-inline");
    let root = repo_root()?;
    let summary = sync::run(&root, all_inline).map_err(|e| e.to_string())?;
    summary.print();
    let status = gitcmd::diff_exit_code(&root)?;
    if status {
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!("check-sync: generated files drifted from .ai/ sources (see `git diff`).");
        Ok(ExitCode::FAILURE)
    }
}

fn cmd_lint_catalog(_rest: &[String]) -> Result<ExitCode, String> {
    let root = repo_root()?;
    let passed = catalog::run(&root).map_err(|e| e.to_string())?;
    if passed {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

/// Resolve the repository root from the compiled crate location, falling back to git.
fn repo_root() -> Result<std::path::PathBuf, String> {
    // crates/xtask/Cargo.toml -> repo root is two levels up from CARGO_MANIFEST_DIR.
    if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
        let p = std::path::Path::new(manifest);
        if let Some(root) = p.parent().and_then(|c| c.parent()) {
            if root.join(".ai").is_dir() {
                return Ok(root.to_path_buf());
            }
        }
    }
    gitcmd::toplevel()
}
