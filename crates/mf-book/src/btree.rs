//! [`BTreeBook`] — the idiomatic Rust starting point: a `BTreeMap<Price, _>` per side, with a
//! time-priority FIFO queue at each price level. `O(log n)` best-price lookup and insert.

use mf_core::{Order, OrderBook, OrderId, Price, Qty, Side};
use std::collections::{BTreeMap, HashMap, VecDeque};

/// A price-time order book backed by a `BTreeMap` per side.
#[derive(Debug, Default)]
pub struct BTreeBook {
    bids: BTreeMap<Price, VecDeque<Order>>,
    asks: BTreeMap<Price, VecDeque<Order>>,
    /// Locates a resting order by id → (side, price), so `reduce`/`get` are direct.
    index: HashMap<OrderId, (Side, Price)>,
}

impl BTreeBook {
    /// An empty book.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn side_map(&self, side: Side) -> &BTreeMap<Price, VecDeque<Order>> {
        match side {
            Side::Bid => &self.bids,
            Side::Ask => &self.asks,
        }
    }
}

impl OrderBook for BTreeBook {
    fn insert(&mut self, order: Order) {
        self.index.insert(order.id, (order.side, order.price));
        let map = match order.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };
        map.entry(order.price).or_default().push_back(order);
    }

    fn best_price(&self, side: Side) -> Option<Price> {
        let map = self.side_map(side);
        match side {
            // Best bid is the highest price; best ask is the lowest.
            Side::Bid => map.keys().next_back().copied(),
            Side::Ask => map.keys().next().copied(),
        }
    }

    fn front_at(&self, side: Side, price: Price) -> Option<(OrderId, Qty)> {
        self.side_map(side)
            .get(&price)
            .and_then(|level| level.front())
            .map(|o| (o.id, o.qty))
    }

    fn level_at(&self, side: Side, price: Price) -> Vec<(OrderId, Qty)> {
        self.side_map(side)
            .get(&price)
            .map(|level| level.iter().map(|o| (o.id, o.qty)).collect())
            .unwrap_or_default()
    }

    fn reduce(&mut self, id: OrderId, qty: Qty) -> bool {
        let Some(&(side, price)) = self.index.get(&id) else {
            return false;
        };
        let map = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };
        let Some(level) = map.get_mut(&price) else {
            return false;
        };
        let Some(pos) = level.iter().position(|o| o.id == id) else {
            return false;
        };
        let order = &mut level[pos];
        order.qty = order.qty - qty;
        let mut removed = false;
        if !order.qty.is_positive() {
            level.remove(pos);
            if level.is_empty() {
                map.remove(&price);
            }
            removed = true;
        }
        if removed {
            self.index.remove(&id);
        }
        true
    }

    fn get(&self, id: OrderId) -> Option<&Order> {
        let &(side, price) = self.index.get(&id)?;
        self.side_map(side).get(&price)?.iter().find(|o| o.id == id)
    }

    fn len(&self) -> usize {
        self.index.len()
    }

    fn depth(&self, side: Side, max_levels: usize) -> Vec<(Price, Qty)> {
        let map = self.side_map(side);
        let level_qty =
            |orders: &VecDeque<Order>| orders.iter().fold(Qty::ZERO, |acc, o| acc + o.qty);
        match side {
            Side::Bid => map
                .iter()
                .rev()
                .take(max_levels)
                .map(|(p, orders)| (*p, level_qty(orders)))
                .collect(),
            Side::Ask => map
                .iter()
                .take(max_levels)
                .map(|(p, orders)| (*p, level_qty(orders)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn ord(id: u64, side: Side, price: i64, qty: i64, ts: u64) -> Order {
        Order::new(
            OrderId(id),
            side,
            Price::new(price.into()),
            Qty::new(qty.into()),
            mf_core::Timestamp(ts),
        )
    }

    #[test]
    fn best_price_picks_top_of_book() {
        let mut b = BTreeBook::new();
        b.insert(ord(1, Side::Bid, 100, 5, 0));
        b.insert(ord(2, Side::Bid, 101, 5, 1));
        b.insert(ord(3, Side::Ask, 105, 5, 2));
        b.insert(ord(4, Side::Ask, 104, 5, 3));
        assert_eq!(b.best_price(Side::Bid), Some(Price::new(dec!(101))));
        assert_eq!(b.best_price(Side::Ask), Some(Price::new(dec!(104))));
        assert_eq!(b.len(), 4);
    }

    #[test]
    fn fifo_time_priority_at_level() {
        let mut b = BTreeBook::new();
        b.insert(ord(1, Side::Bid, 100, 5, 0));
        b.insert(ord(2, Side::Bid, 100, 7, 1));
        assert_eq!(
            b.front_at(Side::Bid, Price::new(dec!(100))),
            Some((OrderId(1), Qty::new(dec!(5))))
        );
        assert_eq!(
            b.level_at(Side::Bid, Price::new(dec!(100))),
            vec![
                (OrderId(1), Qty::new(dec!(5))),
                (OrderId(2), Qty::new(dec!(7)))
            ]
        );
    }

    #[test]
    fn reduce_removes_when_zeroed_and_collapses_empty_level() {
        let mut b = BTreeBook::new();
        b.insert(ord(1, Side::Bid, 100, 5, 0));
        assert!(b.reduce(OrderId(1), Qty::new(dec!(2))));
        assert_eq!(b.get(OrderId(1)).map(|o| o.qty), Some(Qty::new(dec!(3))));
        assert!(b.reduce(OrderId(1), Qty::new(dec!(3))));
        assert_eq!(b.get(OrderId(1)), None);
        assert_eq!(b.best_price(Side::Bid), None);
        assert!(b.is_empty());
        assert!(!b.reduce(OrderId(1), Qty::new(dec!(1))));
    }

    #[test]
    fn depth_aggregates_best_first() {
        let mut b = BTreeBook::new();
        b.insert(ord(1, Side::Bid, 100, 5, 0));
        b.insert(ord(2, Side::Bid, 100, 5, 1));
        b.insert(ord(3, Side::Bid, 99, 3, 2));
        let d = b.depth(Side::Bid, 10);
        assert_eq!(
            d,
            vec![
                (Price::new(dec!(100)), Qty::new(dec!(10))),
                (Price::new(dec!(99)), Qty::new(dec!(3))),
            ]
        );
    }
}
