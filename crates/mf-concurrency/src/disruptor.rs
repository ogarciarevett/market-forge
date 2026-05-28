//! [`DisruptorRunner`] — an LMAX/DMAX-style ring-buffer sequencer.
//!
//! The ring is a power-of-two array of pre-allocated slots. A producer claims the next
//! sequence and publishes an order into `slot[seq & mask]`; a consumer cursor advances through
//! published sequences in order and applies each to the engine. Pre-allocation means the hot
//! path performs no per-order heap allocation, and the single ordered stream keeps matching
//! deterministic.
//!
//! No `unsafe`: slots are `Option<Order>` and the producer/consumer run on one thread
//! (publish-then-drain). The sequencing discipline is identical to a cross-thread Disruptor —
//! a venue can lift the consumer onto its own core without touching matching code.

use mf_core::{MatchingEngine, Order, Trade};

use super::Runner;

/// A pre-allocated ring buffer fronting a matching engine.
#[derive(Debug)]
pub struct DisruptorRunner<E: MatchingEngine> {
    engine: E,
    slots: Vec<Option<Order>>,
    mask: usize,
    /// Highest published sequence (producer cursor); `-1` means nothing published yet.
    published: i64,
    /// Highest consumed sequence.
    consumed: i64,
}

impl<E: MatchingEngine> DisruptorRunner<E> {
    /// Wrap an engine with a ring of `capacity` slots (rounded up to a power of two, min 2).
    pub fn new(engine: E, capacity: usize) -> Self {
        let cap = capacity.max(2).next_power_of_two();
        DisruptorRunner {
            engine,
            slots: vec![None; cap],
            mask: cap - 1,
            published: -1,
            consumed: -1,
        }
    }

    /// Borrow the wrapped engine.
    pub fn engine(&self) -> &E {
        &self.engine
    }

    /// Number of published-but-unconsumed orders currently in the ring.
    pub fn pending(&self) -> usize {
        (self.published - self.consumed) as usize
    }

    /// Publish an order into the ring. Drains first if the ring is full so a single-threaded
    /// caller never blocks.
    fn publish(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        if self.pending() == self.slots.len() {
            trades = self.drain();
        }
        let seq = self.published + 1;
        let idx = (seq as usize) & self.mask;
        self.slots[idx] = Some(order);
        self.published = seq;
        trades
    }

    /// Consume every published-but-unprocessed order in sequence, applying each to the engine.
    fn drain(&mut self) -> Vec<Trade> {
        let mut trades = Vec::new();
        while self.consumed < self.published {
            let seq = self.consumed + 1;
            let idx = (seq as usize) & self.mask;
            if let Some(order) = self.slots[idx].take() {
                trades.extend(self.engine.submit(order));
            }
            self.consumed = seq;
        }
        trades
    }
}

impl<E: MatchingEngine> Runner for DisruptorRunner<E> {
    fn submit(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = self.publish(order);
        trades.extend(self.drain());
        trades
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
    fn disruptor_runner_matches_through_ring() {
        let mut r = DisruptorRunner::new(FifoMatcher::new(BTreeBook::new()), 8);
        assert!(r.submit(ord(1, Side::Ask, 100, 5)).is_empty());
        assert_eq!(r.pending(), 0); // drained each submit
        let trades = r.submit(ord(2, Side::Bid, 100, 5));
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, Qty::new(dec!(5)));
    }

    #[test]
    fn capacity_rounds_up_to_power_of_two() {
        let r = DisruptorRunner::new(FifoMatcher::new(BTreeBook::new()), 5);
        assert_eq!(r.slots.len(), 8);
    }

    #[test]
    fn sequence_order_is_preserved_across_many_orders() {
        // Rest a deep ask book, then sweep it — trades must come out in price-time order
        // regardless of ring wrap-around.
        let mut r = DisruptorRunner::new(FifoMatcher::new(BTreeBook::new()), 4);
        for i in 0..10u64 {
            r.submit(ord(i + 1, Side::Ask, 100, 1));
        }
        let trades = r.submit(ord(100, Side::Bid, 100, 10));
        assert_eq!(trades.len(), 10);
        for (k, t) in trades.iter().enumerate() {
            assert_eq!(t.maker, OrderId(k as u64 + 1)); // oldest first
        }
    }
}
