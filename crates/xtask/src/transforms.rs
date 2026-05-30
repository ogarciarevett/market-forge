//! Pure text transforms, mirroring the TS generator exactly.
//!
//! Parity notes:
//! - JS `String.prototype.trim()` / `trimEnd()` strip the Unicode "white space"
//!   set plus line terminators. Our sources are ASCII, so trimming the standard
//!   ASCII whitespace set (matched by `char::is_whitespace`, which is a superset)
//!   reproduces the same result. We mirror `trim` (both ends) and `trimEnd`.
//! - The frontmatter regex is `^---\n([\s\S]*?)\n---\n?([\s\S]*)$`.

/// JS `String.prototype.trimEnd()` — strip trailing whitespace only.
pub fn trim_end(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_whitespace())
}

/// JS `String.prototype.trim()` — strip leading and trailing whitespace.
pub fn trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace())
}

/// Result of pulling the YAML frontmatter `description` and the markdown body.
pub struct Frontmatter {
    pub description: String,
    pub body: String,
}

/// Mirror of the TS `splitFrontmatter`:
/// regex `^---\n([\s\S]*?)\n---\n?([\s\S]*)$` (non-greedy first group).
/// On no match: `{ description: "", body: text.trim() }`.
pub fn split_frontmatter(text: &str) -> Frontmatter {
    if let Some((fm, body_raw)) = match_frontmatter(text) {
        let mut description = String::new();
        for line in fm.split('\n') {
            if let Some(rest) = line.strip_prefix("description:") {
                // TS: /^description:\s*(.*)$/ — trim leading \s, then trim(), strip quotes.
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

/// Apply the frontmatter regex semantics: require the literal `---\n` prefix,
/// find the first `\n---` close, then an optional single `\n`, and return
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

/// TS: `s.replace(/^["']|["']$/g, "")` — remove a single leading and a single
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

/// TS `tomlBasic`: `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`.
pub fn toml_basic(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// TS `tomlMultiline`: `"""\n${s.replace(/\\/g, "\\\\").replace(/"""/g, '\\"\\"\\"')}\n"""`.
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
