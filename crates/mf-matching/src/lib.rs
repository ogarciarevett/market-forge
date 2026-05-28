//! `mf-matching` — matching algorithms for Market Forge.
//!
//! Each matcher wraps any [`mf_core::OrderBook`] and implements [`mf_core::MatchingEngine`]:
//!
//! - [`FifoMatcher`] — price-time priority (classic CLOB).
//! - [`ProRataMatcher`] — size-proportional allocation at the best level.

#![forbid(unsafe_code)]

mod fifo;
mod prorata;

pub use fifo::FifoMatcher;
pub use prorata::ProRataMatcher;
