//! Interactive prompts (via `inquire`) for any venue dimension not supplied as a flag.

use anyhow::Result;
use inquire::Select;
use mf_codegen::{BookKind, Concurrency, Matching};

/// Prompt for the matching algorithm.
pub fn pick_matching() -> Result<Matching> {
    let choice =
        Select::new("Matching algorithm", vec!["FIFO (price-time)", "Pro-Rata"]).prompt()?;
    Ok(match choice {
        "FIFO (price-time)" => Matching::Fifo,
        "Pro-Rata" => Matching::ProRata,
        other => unreachable!("inquire returned an unlisted matching choice: {other}"),
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
        "BTreeMap" => BookKind::BTreeMap,
        "Bitmap (bounded-tick)" => BookKind::Bitmap,
        other => unreachable!("inquire returned an unlisted book choice: {other}"),
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
        "Single-thread (testing)" => Concurrency::SingleThread,
        "LMAX/DMAX Disruptor" => Concurrency::Disruptor,
        other => unreachable!("inquire returned an unlisted concurrency choice: {other}"),
    })
}
