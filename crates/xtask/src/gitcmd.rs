//! Thin wrappers around external processes (git, bun) and PATH lookup.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Run a command in `cwd`, inheriting stdio. Returns `Ok(true)` on a zero exit code.
pub fn run(program: &str, args: &[String], cwd: &Path) -> Result<bool, String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| format!("failed to spawn `{program}`: {e}"))?;
    Ok(status.success())
}

/// `git diff --exit-code` in `cwd`. Returns `Ok(true)` when the tree is clean
/// (no unstaged changes), `Ok(false)` when it drifted.
pub fn diff_exit_code(cwd: &Path) -> Result<bool, String> {
    let status = Command::new("git")
        .args(["diff", "--exit-code"])
        .current_dir(cwd)
        .status()
        .map_err(|e| format!("failed to spawn git: {e}"))?;
    Ok(status.success())
}

/// `git rev-parse --show-toplevel`.
pub fn toplevel() -> Result<PathBuf, String> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("failed to spawn git: {e}"))?;
    if !out.status.success() {
        return Err("could not determine git repository root".to_string());
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

/// Locate an executable on `PATH` (used to detect bun).
pub fn which(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
