//! The [`MatchingEngine`] trait: submit an order, get the trades it produced.

use crate::types::{Order, Trade};

/// A matching engine. `submit` matches an incoming order against resting liquidity and
/// returns the resulting trades (taker/maker pairs); any unfilled remainder rests in the book.
pub trait MatchingEngine {
    /// Submit an incoming order and return the trades it generated, in execution order.
    fn submit(&mut self, order: Order) -> Vec<Trade>;
}
