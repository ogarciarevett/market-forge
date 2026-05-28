//! `mf-book` — order-book data structures for Market Forge.
//!
//! Both books implement [`mf_core::OrderBook`], so any matching algorithm in `mf-matching`
//! composes with either:
//!
//! - [`BTreeBook`] — `BTreeMap<Price, _>` per side; the idiomatic, unbounded starting point.
//! - [`BitmapBook`] — bounded-tick book with a bitmap of occupied levels for fast best-price.

#![forbid(unsafe_code)]

mod bitmap;
mod btree;

pub use bitmap::{BitmapBook, TickConfig};
pub use btree::BTreeBook;
