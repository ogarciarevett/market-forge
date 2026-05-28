//! The [`OrderBook`] trait: the abstract resting-liquidity store a matching engine drives.
//!
//! Concrete books (BTreeMap, bitmap, …) live in the `mf-book` crate. The trait exposes only
//! the operations a matching algorithm needs, so FIFO and pro-rata can share one book impl.

use crate::types::{Order, OrderId, Price, Qty, Side};

/// A resting-order book. Implementors keep orders sorted by price priority and, within a
/// price level, by time priority (oldest first).
pub trait OrderBook {
    /// Insert a resting order (the remainder of an incoming order that did not fully fill).
    fn insert(&mut self, order: Order);

    /// The best (most aggressive) resting price on `side`: the highest bid or the lowest ask.
    fn best_price(&self, side: Side) -> Option<Price>;

    /// The front (highest time priority) resting order on `side` at `price`, if any.
    ///
    /// Used by FIFO/price-time matching.
    fn front_at(&self, side: Side, price: Price) -> Option<(OrderId, Qty)>;

    /// All resting orders on `side` at `price`, in time priority. Used by pro-rata matching,
    /// which allocates a fill across every order at the level.
    fn level_at(&self, side: Side, price: Price) -> Vec<(OrderId, Qty)>;

    /// Reduce resting order `id` by `qty`, removing it when it reaches zero.
    ///
    /// Returns `true` if the order existed and was reduced/removed.
    fn reduce(&mut self, id: OrderId, qty: Qty) -> bool;

    /// Borrow a resting order by id.
    fn get(&self, id: OrderId) -> Option<&Order>;

    /// Number of resting orders across both sides.
    fn len(&self) -> usize;

    /// `true` when the book holds no resting orders.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Aggregated depth for `side`, best level first, capped at `max_levels`:
    /// `(price, total resting qty at that price)`. Used by visualizations.
    fn depth(&self, side: Side, max_levels: usize) -> Vec<(Price, Qty)>;
}
