//! Pure text transforms for the agent-doc generator.
//!
//! Behavioral notes:
//! - `trim` / `trim_end` strip the Unicode whitespace set (via `char::is_whitespace`)
//!   from both ends / the trailing end respectively. Sources are ASCII, so this is the
//!   plain ASCII-whitespace trim.
//! - Frontmatter is matched by the rule `^---\n([\s\S]*?)\n---\n?([\s\S]*)$`
//!   (non-greedy first group).

/// Strip trailing whitespace only.
pub fn trim_end(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_whitespace())
}

/// Strip leading and trailing whitespace.
pub fn trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace())
}

/// Result of pulling the YAML frontmatter `description` and the markdown body.
pub struct Frontmatter {
    pub description: String,
    pub body: String,
}

/// Pull the YAML-frontmatter `description` and the markdown body, using the rule
/// `^---\n([\s\S]*?)\n---\n?([\s\S]*)$` (non-greedy first group).
/// On no match: empty description, body = `text.trim()`.
pub fn split_frontmatter(text: &str) -> Frontmatter {
    if let Some((fm, body_raw)) = match_frontmatter(text) {
        let mut description = String::new();
        for line in fm.split('\n') {
            if let Some(rest) = line.strip_prefix("description:") {
                // `^description:\s*(.*)$` — drop leading whitespace, trim, strip quotes.
                let val = rest.trim_start_matches(|c: char| c.is_whitespace());
                description = strip_surrounding_quotes(trim(val));
            }
        }
        Frontmatter {
            description,
            body: trim(body_raw).to_string(),
        }
    } else {
        Frontmatter {
            description: String::new(),
            body: trim(text).to_string(),
        }
    }
}

/// Apply the frontmatter rule: require the literal `---\n` prefix, find the first
/// `\n---` close, then an optional single `\n`, and return
/// `(frontmatter_inner, body_remainder)`.
fn match_frontmatter(text: &str) -> Option<(&str, &str)> {
    let after_open = text.strip_prefix("---\n")?;
    // Non-greedy first group: smallest inner up to the first `\n---`.
    let close = after_open.find("\n---")?;
    let fm = &after_open[..close];
    let after_close = &after_open[close + "\n---".len()..];
    // `\n---\n?` — consume one optional trailing newline.
    let body = after_close.strip_prefix('\n').unwrap_or(after_close);
    Some((fm, body))
}

/// Rule `s.replace(/^["']|["']$/g, "")` — remove a single leading and a single
/// trailing quote (`"` or `'`) independently.
fn strip_surrounding_quotes(s: &str) -> String {
    let mut out = s;
    if out.starts_with('"') || out.starts_with('\'') {
        out = &out[1..];
    }
    if out.ends_with('"') || out.ends_with('\'') {
        out = &out[..out.len() - 1];
    }
    out.to_string()
}

/// Encode `s` as a TOML basic string: escape `\` then `"`, and wrap in double quotes.
pub fn toml_basic(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Encode `s` as a TOML multiline string: escape `\` and any `"""`, and wrap in `"""…"""`.
pub fn toml_multiline(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"");
    format!("\"\"\"\n{escaped}\n\"\"\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_extracts_description_and_body() {
        let src = "---\ndescription: Hello world\nallowed-tools: Read\n---\n\n# Body\ntext\n";
        let fm = split_frontmatter(src);
        assert_eq!(fm.description, "Hello world");
        assert_eq!(fm.body, "# Body\ntext");
    }

    #[test]
    fn frontmatter_strips_quotes() {
        let src = "---\ndescription: \"quoted value\"\n---\nbody\n";
        let fm = split_frontmatter(src);
        assert_eq!(fm.description, "quoted value");
    }

    #[test]
    fn no_frontmatter_returns_trimmed_body() {
        let src = "\n\n# Just a body\n\n";
        let fm = split_frontmatter(src);
        assert_eq!(fm.description, "");
        assert_eq!(fm.body, "# Just a body");
    }

    #[test]
    fn frontmatter_non_greedy_first_close() {
        // Body itself contains a `---` line; the first close must win.
        let src = "---\ndescription: d\n---\nbody line\n---\nmore\n";
        let fm = split_frontmatter(src);
        assert_eq!(fm.description, "d");
        assert_eq!(fm.body, "body line\n---\nmore");
    }

    #[test]
    fn toml_basic_escapes_backslash_then_quote() {
        assert_eq!(toml_basic(r#"a\b"c"#), "\"a\\\\b\\\"c\"");
    }

    #[test]
    fn toml_multiline_wraps_and_escapes() {
        assert_eq!(toml_multiline("x"), "\"\"\"\nx\n\"\"\"");
        assert_eq!(toml_multiline("\"\"\""), "\"\"\"\n\\\"\\\"\\\"\n\"\"\"");
    }
}
