//! `lint-catalog` — structural lint for the algorithm catalog.
//!
//! Every entry under `docs/catalog/**/*.md` (except `_index.md` / `README.md`) must carry a
//! level-1 title, the required section markers, and a mermaid diagram, per `docs/standards.md`
//! §2. Replaces the former `scripts/lint-catalog.sh` so the project tooling is pure Rust.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Literal markers every catalog entry must contain (see `docs/standards.md` §2).
const REQUIRED_MARKERS: &[&str] = &[
    "```mermaid",
    "**What it is.**",
    "**When to pick this.**",
    "**When NOT to pick this.**",
    "**Real venue.**",
    "**Recommended crate.**",
];

/// Prose past this word count earns a WARN (the style cap is ≤300; the slack covers diagram
/// source + table tokens). Never fails the lint.
const WORD_WARN_LIMIT: usize = 600;

/// Lint every catalog entry under `<root>/docs/catalog`. Returns `Ok(true)` when all entries
/// pass, `Ok(false)` when at least one is missing a required title or marker.
pub fn run(root: &Path) -> io::Result<bool> {
    let catalog = root.join("docs/catalog");
    if !catalog.is_dir() {
        println!("lint-catalog: no docs/catalog/ directory yet — nothing to lint.");
        return Ok(true);
    }

    let mut entries = Vec::new();
    collect_markdown(&catalog, &mut entries)?;
    entries.sort();

    let mut failed = false;
    let mut count = 0usize;
    for file in &entries {
        let base = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if base == "_index.md" || base == "README.md" {
            continue;
        }
        count += 1;

        let content = fs::read_to_string(file)?;
        let rel = file.strip_prefix(root).unwrap_or(file).display();

        if !has_level1_title(&content) {
            println!("FAIL  {rel} — missing level-1 '# Title'");
            failed = true;
        }
        for &marker in REQUIRED_MARKERS {
            if !content.contains(marker) {
                println!("FAIL  {rel} — missing marker: {marker}");
                failed = true;
            }
        }

        let words = content.split_whitespace().count();
        if words > WORD_WARN_LIMIT {
            println!("WARN  {rel} — {words} words (prose should be ≤300; trim or split)");
        }
    }

    println!("lint-catalog: checked {count} catalog entries.");
    if failed {
        println!("lint-catalog: FAILED — fix the entries above.");
        return Ok(false);
    }
    println!("lint-catalog: OK");
    Ok(true)
}

/// True if any of the first 20 lines matches `^# .+` (a level-1 heading with text).
fn has_level1_title(content: &str) -> bool {
    content
        .lines()
        .take(20)
        .any(|line| line.strip_prefix("# ").is_some_and(|rest| !rest.is_empty()))
}

/// Recursively collect `*.md` files under `dir`.
fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_level1_title() {
        assert!(has_level1_title("# FIFO matching\n\nbody"));
        assert!(has_level1_title("front matter\n# Title on line two"));
    }

    #[test]
    fn rejects_missing_or_empty_title() {
        assert!(!has_level1_title("## subheading only\n\nno level-1 here"));
        assert!(!has_level1_title("# \nempty title"));
    }

    #[test]
    fn title_must_be_within_first_20_lines() {
        let mut doc = String::new();
        for _ in 0..25 {
            doc.push_str("filler\n");
        }
        doc.push_str("# Late title\n");
        assert!(!has_level1_title(&doc));
    }
}
