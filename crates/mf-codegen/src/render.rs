//! Render a [`VenueSpec`] into a generated Cargo workspace using Tera templates.

use crate::error::CodegenError;
use crate::matrix;
use crate::spec::{BookKind, Concurrency, Matching, VenueSpec};
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

// Templates are embedded so the installed binary is self-contained.
const T_WS_CARGO: &str = include_str!("../../../templates/venue/Cargo.toml.tera");
const T_README: &str = include_str!("../../../templates/venue/README.md.tera");
const T_ENGINE_CARGO: &str = include_str!("../../../templates/venue/engine/Cargo.toml.tera");
const T_ENGINE_LIB: &str = include_str!("../../../templates/venue/engine/lib.rs.tera");
const T_ENGINE_GOLDEN: &str = include_str!("../../../templates/venue/engine/golden_test.rs.tera");
const T_ENGINE_BENCH: &str = include_str!("../../../templates/venue/engine/bench.rs.tera");

struct Choice {
    matcher_type: &'static str,
    matching_tag: &'static str,
    matching_label: &'static str,
    matching_why: &'static str,
    book_type: &'static str,
    book_ctor: &'static str,
    book_label: &'static str,
    book_why: &'static str,
    runner_type: &'static str,
    runner_ctor: &'static str,
    concurrency_label: &'static str,
    concurrency_why: &'static str,
}

fn choices(spec: &VenueSpec) -> Choice {
    let (matcher_type, matching_tag, matching_label, matching_why) = match spec.matching {
        Matching::Fifo => (
            "FifoMatcher",
            "fifo",
            "FIFO (price-time)",
            "the classic CLOB discipline used by most equities and crypto venues",
        ),
        Matching::ProRata => (
            "ProRataMatcher",
            "prorata",
            "Pro-Rata",
            "rewards large resting liquidity; common in futures/options venues",
        ),
    };
    let (book_type, book_ctor, book_label, book_why) = match spec.book {
        BookKind::BTreeMap => (
            "BTreeBook",
            "BTreeBook::new()",
            "BTreeMap",
            "idiomatic, unbounded, O(log n) best-price",
        ),
        BookKind::Bitmap => (
            "BitmapBook",
            "BitmapBook::default()",
            "Bitmap (bounded-tick)",
            "O(1)-amortized best-price when the price range is bounded and known",
        ),
    };
    let (runner_type, runner_ctor, concurrency_label, concurrency_why) = match spec.concurrency {
        Concurrency::SingleThread => (
            "SingleThreadRunner",
            "SingleThreadRunner::new(matcher)",
            "Single-thread",
            "simplest and fully deterministic; ideal for tests and backtests",
        ),
        Concurrency::Disruptor => (
            "DisruptorRunner",
            "DisruptorRunner::new(matcher, 1024)",
            "LMAX/DMAX Disruptor",
            "pre-allocated ring buffer with a sequence cursor; no per-order allocation",
        ),
    };
    Choice {
        matcher_type,
        matching_tag,
        matching_label,
        matching_why,
        book_type,
        book_ctor,
        book_label,
        book_why,
        runner_type,
        runner_ctor,
        concurrency_label,
        concurrency_why,
    }
}

fn context(spec: &VenueSpec, sdk_path: &Path) -> Context {
    let c = choices(spec);
    let mut ctx = Context::new();
    ctx.insert("crate_name", &spec.name);
    ctx.insert("crate_ident", &spec.crate_ident());
    ctx.insert("sdk_path", &sdk_path.display().to_string());
    ctx.insert("matching", c.matching_tag);
    ctx.insert("matching_label", c.matching_label);
    ctx.insert("matching_why", c.matching_why);
    ctx.insert("book_label", c.book_label);
    ctx.insert("book_why", c.book_why);
    ctx.insert("concurrency_label", c.concurrency_label);
    ctx.insert("concurrency_why", c.concurrency_why);
    ctx.insert("matcher_type", c.matcher_type);
    ctx.insert("book_type", c.book_type);
    ctx.insert("book_ctor", c.book_ctor);
    ctx.insert("runner_type", c.runner_type);
    ctx.insert("runner_ctor", c.runner_ctor);
    ctx
}

fn tera() -> Result<Tera, CodegenError> {
    let mut t = Tera::default();
    t.add_raw_templates(vec![
        ("ws_cargo", T_WS_CARGO),
        ("readme", T_README),
        ("engine_cargo", T_ENGINE_CARGO),
        ("engine_lib", T_ENGINE_LIB),
        ("engine_golden", T_ENGINE_GOLDEN),
        ("engine_bench", T_ENGINE_BENCH),
    ])?;
    Ok(t)
}

fn write(path: PathBuf, contents: &str) -> Result<PathBuf, CodegenError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CodegenError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    std::fs::write(&path, contents).map_err(|source| CodegenError::Io {
        path: path.display().to_string(),
        source,
    })?;
    Ok(path)
}

/// Render `spec` into a new workspace rooted at `out_dir`, with the engine crate depending on
/// the Market Forge SDK crates found under `sdk_path`. Returns the paths written.
///
/// `out_dir` is created if missing. Validates the name and compatibility matrix first.
pub fn render_venue(
    spec: &VenueSpec,
    out_dir: &Path,
    sdk_path: &Path,
) -> Result<Vec<PathBuf>, CodegenError> {
    VenueSpec::validate_name(&spec.name)?;
    matrix::check(spec)?;

    let t = tera()?;
    let ctx = context(spec, sdk_path);
    let engine = out_dir.join("crates").join("engine");

    let files = [
        (out_dir.join("Cargo.toml"), "ws_cargo"),
        (out_dir.join("README.md"), "readme"),
        (engine.join("Cargo.toml"), "engine_cargo"),
        (engine.join("src").join("lib.rs"), "engine_lib"),
        (engine.join("tests").join("golden.rs"), "engine_golden"),
        (engine.join("benches").join("match.rs"), "engine_bench"),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (path, name) in files {
        let rendered = t.render(name, &ctx)?;
        written.push(write(path, &rendered)?);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> VenueSpec {
        VenueSpec {
            name: "my-clob".into(),
            matching: Matching::Fifo,
            book: BookKind::BTreeMap,
            concurrency: Concurrency::SingleThread,
        }
    }

    #[test]
    fn renders_expected_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let written = render_venue(&spec(), tmp.path(), Path::new("/sdk")).unwrap();
        assert_eq!(written.len(), 6);
        let lib = std::fs::read_to_string(tmp.path().join("crates/engine/src/lib.rs")).unwrap();
        assert!(lib.contains("FifoMatcher<BTreeBook>"));
        assert!(lib.contains("SingleThreadRunner"));
        let cargo = std::fs::read_to_string(tmp.path().join("crates/engine/Cargo.toml")).unwrap();
        assert!(cargo.contains("/sdk/mf-core"));
        let golden =
            std::fs::read_to_string(tmp.path().join("crates/engine/tests/golden.rs")).unwrap();
        assert!(golden.contains("fifo_fills_oldest_resting_order_first"));
    }

    #[test]
    fn rejects_invalid_name() {
        let mut s = spec();
        s.name = "Bad Name".into();
        let tmp = tempfile::tempdir().unwrap();
        assert!(render_venue(&s, tmp.path(), Path::new("/sdk")).is_err());
    }
}
