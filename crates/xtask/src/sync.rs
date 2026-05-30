//! Core generator — materializes every per-tool agent artifact from the
//! hand-edited sources under `.ai/` (the project's single source of truth).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::transforms::{split_frontmatter, toml_basic, toml_multiline, trim, trim_end};

/// Counts surfaced in the final summary (printed after a sync run).
pub struct Summary {
    all_inline: bool,
    commands: usize,
    agents: usize,
    skills: usize,
    workflows: usize,
}

impl Summary {
    pub fn print(&self) {
        if self.all_inline {
            println!("sync-ai: regenerated (contract forced --all-inline)");
        } else {
            println!("sync-ai: regenerated (AGENTS/cursor inline; CLAUDE/GEMINI @import stubs)");
        }
        println!(
            "  docs    \u{2192} AGENTS.md, CLAUDE.md, GEMINI.md, .ai/generated/rules.mdc (+ .cursor symlink)"
        );
        println!(
            "  commands\u{2192} {} \u{00d7} {{.claude, .opencode (md), .gemini (toml)}}",
            self.commands
        );
        println!(
            "  agents  \u{2192} {} \u{00d7} {{.claude, .opencode, .gemini}}",
            self.agents
        );
        println!(
            "  skills  \u{2192} {} \u{00d7} {{.claude/skills, .gemini/skills}}",
            self.skills
        );
        println!(
            "  workflows\u{2192} {} in .ai/workflows/ (Claude-Code-only, by path)",
            self.workflows
        );
    }
}

/// Run the full sync against `root` (repo root). `all_inline` forces the
/// CLAUDE/GEMINI stubs to inline the contract too.
pub fn run(root: &Path, all_inline: bool) -> io::Result<Summary> {
    let ai = root.join(".ai");
    let generated_dir = ai.join("generated");
    let templates = ai.join("templates");
    let memory = ai.join("memory.md");

    // ---------------------------------------------------------- contract docs
    // Seed the local, gitignored memory log from the committed template if absent.
    if !memory.exists() {
        fs::copy(ai.join("memory.example.md"), &memory)?;
    }

    let banner = trim(&read(&templates.join("banner.md"))?).to_string();
    let cursor_header = trim(&read(&templates.join("cursor.header.mdc"))?).to_string();
    let context = trim_end(&read(&ai.join("context.md"))?).to_string();
    let pipeline = trim_end(&read(&ai.join("pipeline.md"))?).to_string();

    let contract_inline = format!("{context}\n\n{pipeline}");
    let contract_imports = "@.ai/context.md\n\n@.ai/pipeline.md";
    let memory_section = [
        "## Memory",
        "",
        "Shared working log: `.ai/memory.md` \u{2014} LOCAL and gitignored (seed from",
        "`.ai/memory.example.md`; `cargo xtask sync-ai` seeds it for you). It is not inlined here;",
        "tools that resolve imports pull it in, and opencode reads it directly:",
        "",
        "@.ai/memory.md",
    ]
    .join("\n");

    for target in doc_targets(root, &generated_dir) {
        let contract: &str = if target.mode == Mode::Imports && !all_inline {
            contract_imports
        } else {
            &contract_inline
        };
        let body = format!("{contract}\n\n{memory_section}");
        let out = match target.kind {
            Kind::Cursor => format!("{cursor_header}\n\n{banner}\n\n{body}\n"),
            Kind::Markdown => format!("{banner}\n\n{body}\n"),
        };
        write_real(&target.path, &out)?;
    }
    ensure_symlink(
        &root.join(".cursor/rules/00-context.mdc"),
        &generated_dir.join("rules.mdc"),
    )?;

    // ------------------------------------------------------- per-tool assets
    let commands_src = ai.join("commands");
    let agents_src = ai.join("agents");
    let skills_src = ai.join("skills");

    // Commands -> Claude/opencode (copy) + Gemini (md -> toml).
    let mut commands = 0usize;
    for name in md_names(&commands_src)? {
        let raw = read(&commands_src.join(format!("{name}.md")))?;
        write_real(
            &root.join(".claude/commands").join(format!("{name}.md")),
            &raw,
        )?;
        write_real(
            &root.join(".opencode/commands").join(format!("{name}.md")),
            &raw,
        )?;
        let fm = split_frontmatter(&raw);
        let description = if fm.description.is_empty() {
            format!("{name} command")
        } else {
            fm.description
        };
        let toml = format!(
            "description = {}\nprompt = {}\n",
            toml_basic(&description),
            toml_multiline(&fm.body)
        );
        write_real(
            &root.join(".gemini/commands").join(format!("{name}.toml")),
            &toml,
        )?;
        commands += 1;
    }

    // Agents -> Claude/opencode/Gemini (copy).
    let mut agents = 0usize;
    for name in md_names(&agents_src)? {
        let raw = read(&agents_src.join(format!("{name}.md")))?;
        write_real(
            &root.join(".claude/agents").join(format!("{name}.md")),
            &raw,
        )?;
        write_real(
            &root.join(".opencode/agents").join(format!("{name}.md")),
            &raw,
        )?;
        write_real(
            &root.join(".gemini/agents").join(format!("{name}.md")),
            &raw,
        )?;
        agents += 1;
    }

    // Skills -> .claude/skills + .gemini/skills (one dir per skill).
    let mut skills = 0usize;
    if skills_src.is_dir() {
        for name in sorted_dir_entries(&skills_src)? {
            let src = skills_src.join(&name);
            if !src.is_dir() {
                continue;
            }
            copy_dir_fresh(&src, &root.join(".claude/skills").join(&name))?;
            copy_dir_fresh(&src, &root.join(".gemini/skills").join(&name))?;
            skills += 1;
        }
    }

    // Workflows -> referenced in place (Claude-Code-only). Counted only.
    let workflows_src = ai.join("workflows");
    let workflows = if workflows_src.is_dir() {
        sorted_dir_entries(&workflows_src)?
            .iter()
            .filter(|f| f.ends_with(".js"))
            .count()
    } else {
        0
    };

    Ok(Summary {
        all_inline,
        commands,
        agents,
        skills,
        workflows,
    })
}

#[derive(PartialEq, Eq)]
enum Kind {
    Markdown,
    Cursor,
}

#[derive(PartialEq, Eq)]
enum Mode {
    Inline,
    Imports,
}

struct DocTarget {
    path: PathBuf,
    kind: Kind,
    mode: Mode,
}

fn doc_targets(root: &Path, generated_dir: &Path) -> Vec<DocTarget> {
    vec![
        DocTarget {
            path: root.join("AGENTS.md"),
            kind: Kind::Markdown,
            mode: Mode::Inline,
        },
        DocTarget {
            path: generated_dir.join("rules.mdc"),
            kind: Kind::Cursor,
            mode: Mode::Inline,
        },
        DocTarget {
            path: root.join("CLAUDE.md"),
            kind: Kind::Markdown,
            mode: Mode::Imports,
        },
        DocTarget {
            path: root.join("GEMINI.md"),
            kind: Kind::Markdown,
            mode: Mode::Imports,
        },
    ]
}

fn read(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

/// `mkdir -p` parent, drop a pre-existing symlink, then write the file.
fn write_real(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if is_symlink(path) {
        fs::remove_file(path)?;
    }
    fs::write(path, content)
}

/// `mkdir -p` parent, remove any existing entry, then create a relative symlink
/// (`relative(dirname(link), target)`).
fn ensure_symlink(link_path: &Path, target_path: &Path) -> io::Result<()> {
    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if link_path.exists() || is_symlink(link_path) {
        // `force: true` — ignore "not found".
        match fs::remove_file(link_path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    let link_dir = link_path.parent().unwrap_or_else(|| Path::new(""));
    let rel = relative_path(link_dir, target_path);
    symlink_file(&rel, link_path)
}

#[cfg(unix)]
fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Markdown basenames (without `.md`) in `dir`, excluding `README.md`, sorted.
fn md_names(dir: &Path) -> io::Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name.ends_with(".md") && name != "README.md" {
            names.push(name[..name.len() - 3].to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// Directory entry names (files + dirs), sorted for determinism.
fn sorted_dir_entries(dir: &Path) -> io::Result<Vec<String>> {
    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        names.push(entry.file_name().to_string_lossy().to_string());
    }
    names.sort();
    Ok(names)
}

/// `rm -rf` dest, `mkdir -p` parent, then recursively copy src -> dest.
fn copy_dir_fresh(src: &Path, dest: &Path) -> io::Result<()> {
    if dest.exists() || is_symlink(dest) {
        match fs::remove_dir_all(dest) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                // dest might be a symlink to a file or a plain file.
                if fs::remove_file(dest).is_err() {
                    return Err(e);
                }
            }
        }
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_dir_recursive(src, dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if file_type.is_symlink() {
            let link_target = fs::read_link(&from)?;
            if to.exists() || is_symlink(&to) {
                let _ = fs::remove_file(&to);
            }
            symlink_file(&link_target, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Pure path-relative computation matching Node's `path.relative(from, to)` for
/// the inputs we use (both absolute, same root). Produces e.g.
/// `../../.ai/generated/rules.mdc`.
fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_comps: Vec<_> = normalized_components(from);
    let to_comps: Vec<_> = normalized_components(to);

    let mut i = 0;
    while i < from_comps.len() && i < to_comps.len() && from_comps[i] == to_comps[i] {
        i += 1;
    }

    let mut result = PathBuf::new();
    for _ in i..from_comps.len() {
        result.push("..");
    }
    for comp in &to_comps[i..] {
        result.push(comp);
    }
    result
}

fn normalized_components(p: &Path) -> Vec<String> {
    use std::path::Component;
    let mut out: Vec<String> = Vec::new();
    for comp in p.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(s) => out.push(s.to_string_lossy().to_string()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_cursor_symlink() {
        let from = Path::new("/repo/.cursor/rules");
        let to = Path::new("/repo/.ai/generated/rules.mdc");
        assert_eq!(
            relative_path(from, to),
            PathBuf::from("../../.ai/generated/rules.mdc")
        );
    }
}
