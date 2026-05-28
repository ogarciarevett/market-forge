//! `mf-concurrency` — concurrency runners for Market Forge.
//!
//! A [`Runner`] wraps a [`mf_core::MatchingEngine`] and decides *how* submitted orders reach
//! it. Both runners keep the matching path single-writer (the engine is never shared), which
//! is what makes a matching engine deterministic and fast.
//!
//! - [`SingleThreadRunner`] — apply each order inline. Simplest; ideal for tests/backtests.
//! - [`DisruptorRunner`] — an LMAX/DMAX-style pre-allocated ring buffer with a sequence
//!   cursor. Orders are *published* into the ring and a consumer cursor drains them in
//!   sequence. This is the structure the original OrderBook-rs design used; here it runs the
//!   producer and consumer on one thread (publish-then-drain) so behavior is deterministic and
//!   testable. A generated venue can move the consumer to its own core without changing the
//!   matching code — the engine only ever sees a single, ordered stream.

#![forbid(unsafe_code)]

mod disruptor;

pub use disruptor::DisruptorRunner;

use mf_core::{MatchingEngine, Order, Trade};

/// How submitted orders are delivered to the wrapped matching engine.
pub trait Runner {
    /// Submit an order and return the trades produced once it has been processed.
    fn submit(&mut self, order: Order) -> Vec<Trade>;
}

/// Applies each order to the engine inline, on the calling thread.
#[derive(Debug)]
pub struct SingleThreadRunner<E: MatchingEngine> {
    engine: E,
}

impl<E: MatchingEngine> SingleThreadRunner<E> {
    /// Wrap a matching engine.
    pub fn new(engine: E) -> Self {
        SingleThreadRunner { engine }
    }

    /// Borrow the wrapped engine.
    pub fn engine(&self) -> &E {
        &self.engine
    }
}

impl<E: MatchingEngine> Runner for SingleThreadRunner<E> {
    fn submit(&mut self, order: Order) -> Vec<Trade> {
        self.engine.submit(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_book::BTreeBook;
    use mf_core::{Order, OrderId, Price, Qty, Side, Timestamp};
    use mf_matching::FifoMatcher;
    use rust_decimal_macros::dec;

    fn ord(id: u64, side: Side, price: i64, qty: i64) -> Order {
        Order::new(
            OrderId(id),
            side,
            Price::new(price.into()),
            Qty::new(qty.into()),
            Timestamp(0),
        )
    }

    #[test]
    fn single_thread_runner_matches_inline() {
        let mut r = SingleThreadRunner::new(FifoMatcher::new(BTreeBook::new()));
        assert!(r.submit(ord(1, Side::Ask, 100, 5)).is_empty());
        let trades = r.submit(ord(2, Side::Bid, 100, 5));
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, Qty::new(dec!(5)));
    }
}
