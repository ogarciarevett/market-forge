//! Thin wrappers around git invoked as an external process.

use std::path::{Path, PathBuf};
use std::process::Command;

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
