//! `market-forge` — an interactive CLI that generates a tailored matching-engine venue.
//!
//! `market-forge new my-venue` runs a short wizard (or takes `--matching/--book/--concurrency`
//! flags) and stamps out a Cargo workspace that builds, tests, and benches.

mod wizard;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mf_codegen::{matrix, render_venue, BookKind, Concurrency, Matching, VenueSpec};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "market-forge",
    version,
    about = "Generate a matching-engine venue from your design choices."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new venue workspace.
    New(NewArgs),
    /// List the available algorithm choices (and where the full catalog lives).
    List {
        /// Optional category filter (matching | book | concurrency).
        category: Option<String>,
    },
    /// Validate a venue spec TOML file without generating anything.
    Validate {
        /// Path to a spec.toml describing the venue.
        path: PathBuf,
    },
}

#[derive(clap::Args)]
struct NewArgs {
    /// Venue name (lowercase, hyphenated) — also the output directory.
    name: String,
    /// Matching algorithm: fifo | pro-rata. Prompts if omitted.
    #[arg(long)]
    matching: Option<String>,
    /// Book data structure: btreemap | bitmap. Prompts if omitted.
    #[arg(long)]
    book: Option<String>,
    /// Concurrency model: single-thread | disruptor. Prompts if omitted.
    #[arg(long)]
    concurrency: Option<String>,
    /// Output directory (defaults to ./<name>).
    #[arg(long)]
    out: Option<PathBuf>,
    /// Path to the Market Forge SDK crates dir (defaults to the bundled checkout).
    #[arg(long)]
    sdk_path: Option<PathBuf>,
}

/// Where the SDK crates live by default: the `crates/` dir of this checkout.
fn default_sdk_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("crates"))
}

fn resolve_spec(args: &NewArgs) -> Result<VenueSpec> {
    VenueSpec::validate_name(&args.name).context("invalid venue name")?;
    let matching = match &args.matching {
        Some(s) => s.parse::<Matching>()?,
        None => wizard::pick_matching()?,
    };
    let book = match &args.book {
        Some(s) => s.parse::<BookKind>()?,
        None => wizard::pick_book()?,
    };
    let concurrency = match &args.concurrency {
        Some(s) => s.parse::<Concurrency>()?,
        None => wizard::pick_concurrency()?,
    };
    Ok(VenueSpec {
        name: args.name.clone(),
        matching,
        book,
        concurrency,
    })
}

fn run_new(args: NewArgs) -> Result<()> {
    let spec = resolve_spec(&args)?;

    if let matrix::Compatibility::Workable(note) = matrix::evaluate(&spec) {
        eprintln!("note: {note}");
    }

    let out_dir = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from(&spec.name));
    let sdk_path = args.sdk_path.clone().unwrap_or_else(default_sdk_path);

    let written = render_venue(&spec, &out_dir, &sdk_path)
        .with_context(|| format!("failed to generate venue in {}", out_dir.display()))?;

    println!(
        "✔ Generated {} ({} files)",
        out_dir.display(),
        written.len()
    );
    println!("\nNext steps:");
    println!("  cd {}", out_dir.display());
    println!("  cargo build --release");
    println!("  cargo test");
    println!("  cargo bench");
    Ok(())
}

fn run_list(category: Option<String>) -> Result<()> {
    let cat = category.unwrap_or_default().to_ascii_lowercase();
    let show = |c: &str| cat.is_empty() || cat == c;
    if show("matching") {
        println!("matching:    fifo, pro-rata");
    }
    if show("book") {
        println!("book:        btreemap, bitmap");
    }
    if show("concurrency") {
        println!("concurrency: single-thread, disruptor");
    }
    println!("\nFull algorithm catalog (Mermaid + plain-language): docs/catalog/");
    Ok(())
}

fn run_validate(path: PathBuf) -> Result<()> {
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let spec: VenueSpec =
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
    VenueSpec::validate_name(&spec.name)?;
    matrix::check(&spec)?;
    match matrix::evaluate(&spec) {
        matrix::Compatibility::Workable(note) => println!("OK (workable): {note}"),
        _ => println!("OK (natural): {} is coherent", spec.name),
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::New(args) => run_new(args),
        Command::List { category } => run_list(category),
        Command::Validate { path } => run_validate(path),
    }
}
