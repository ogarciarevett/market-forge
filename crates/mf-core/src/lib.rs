//! `mf-core` — Market Forge core domain types and matching-engine traits.
//!
//! This crate is the contract every other Market Forge crate depends on: the value types
//! ([`Order`], [`Trade`], [`Price`], [`Qty`], …) and the two traits that let matching
//! algorithms ([`MatchingEngine`]) and book data structures ([`OrderBook`]) be mixed and
//! matched by the generator.
//!
//! All money and size use [`rust_decimal::Decimal`]; never `f64`.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

mod book;
mod engine;
mod types;

pub use book::OrderBook;
pub use engine::MatchingEngine;
pub use types::{Order, OrderId, Price, Qty, Side, Timestamp, Trade};
