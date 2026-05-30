//! xtask — dev-only task runner for market-forge.
//!
//! Subcommands:
//!   sync-ai [--all-inline]  Regenerate every per-tool agent artifact from `.ai/`,
//!                           byte-for-byte identical to `scripts/sync-ai-docs.ts`.
//!   check-sync              Run `sync-ai`, then `git diff --exit-code` (fails on drift).
//!   parity                  Run the Rust sync, then the TS generator via bun (if present),
//!                           and assert the working tree is unchanged.
//!
//! The TS generator (`scripts/sync-ai-docs.ts`) remains the portable cross-ai-template
//! reference; this binary is a faithful Rust port whose output must match it exactly.

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
        "parity" => cmd_parity(rest),
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
        "xtask — dev task runner (Rust port of scripts/sync-ai-docs.ts)\n\n\
         USAGE:\n\
         \x20 cargo xtask sync-ai [--all-inline]   Regenerate per-tool agent configs from .ai/\n\
         \x20 cargo xtask check-sync               Sync then `git diff --exit-code`\n\
         \x20 cargo xtask parity                   Sync (Rust), run bun TS gen, assert no drift\n"
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

fn cmd_parity(rest: &[String]) -> Result<ExitCode, String> {
    let all_inline = rest.iter().any(|a| a == "--all-inline");
    let root = repo_root()?;

    // 1) Rust sync.
    sync::run(&root, all_inline).map_err(|e| e.to_string())?;

    // 2) TS sync via bun, if available.
    if gitcmd::which("bun").is_none() {
        println!("parity: bun not found on PATH — skipping TS cross-check (exit 0).");
        return Ok(ExitCode::SUCCESS);
    }

    let mut bun_args = vec!["scripts/sync-ai-docs.ts".to_string()];
    if all_inline {
        bun_args.push("--all-inline".to_string());
    }
    let ok = gitcmd::run("bun", &bun_args, &root)?;
    if !ok {
        return Err("parity: `bun scripts/sync-ai-docs.ts` failed".to_string());
    }

    // 3) Assert tree unchanged after the TS run (Rust output already matched it).
    let clean = gitcmd::diff_exit_code(&root)?;
    if clean {
        println!("parity: PASS — Rust and TS generators produce an identical tree.");
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!("parity: MISS — TS generator changed files the Rust port did not match.");
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
