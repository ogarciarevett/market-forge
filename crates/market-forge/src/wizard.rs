//! Interactive prompts (via `inquire`) for any venue dimension not supplied as a flag.

use anyhow::Result;
use inquire::Select;
use mf_codegen::{BookKind, Concurrency, Matching};

/// Prompt for the matching algorithm.
pub fn pick_matching() -> Result<Matching> {
    let choice =
        Select::new("Matching algorithm", vec!["FIFO (price-time)", "Pro-Rata"]).prompt()?;
    Ok(match choice {
        "Pro-Rata" => Matching::ProRata,
        _ => Matching::Fifo,
    })
}

/// Prompt for the book data structure.
pub fn pick_book() -> Result<BookKind> {
    let choice = Select::new(
        "Book data structure",
        vec!["BTreeMap", "Bitmap (bounded-tick)"],
    )
    .prompt()?;
    Ok(match choice {
        "Bitmap (bounded-tick)" => BookKind::Bitmap,
        _ => BookKind::BTreeMap,
    })
}

/// Prompt for the concurrency model.
pub fn pick_concurrency() -> Result<Concurrency> {
    let choice = Select::new(
        "Concurrency model",
        vec!["Single-thread (testing)", "LMAX/DMAX Disruptor"],
    )
    .prompt()?;
    Ok(match choice {
        "LMAX/DMAX Disruptor" => Concurrency::Disruptor,
        _ => Concurrency::SingleThread,
    })
}
