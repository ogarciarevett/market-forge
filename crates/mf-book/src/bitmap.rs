//! [`BitmapBook`] — a bounded-tick book that uses a bitmap of occupied price levels for
//! `O(1)`-amortized best-price lookup (find the highest/lowest set bit), with a sparse map of
//! time-priority queues for storage. Real venues use this when the price range is bounded and
//! known (e.g. tick-bounded equity/options books).

use mf_core::{Order, OrderBook, OrderId, Price, Qty, Side};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::{HashMap, VecDeque};

/// Bounded-tick configuration: prices live on `[min, min + tick*ticks)` aligned to `tick`.
#[derive(Debug, Clone, Copy)]
pub struct TickConfig {
    pub min_price: Decimal,
    pub tick_size: Decimal,
    pub num_ticks: u32,
}

impl Default for TickConfig {
    /// Prices on `[0, 1000)` in `0.01` ticks — a reasonable demo default.
    fn default() -> Self {
        TickConfig {
            min_price: Decimal::ZERO,
            tick_size: Decimal::new(1, 2), // 0.01
            num_ticks: 100_000,
        }
    }
}

/// A bitmap-backed, bounded-tick order book.
#[derive(Debug)]
pub struct BitmapBook {
    cfg: TickConfig,
    bid_bits: Vec<u64>,
    ask_bits: Vec<u64>,
    bids: HashMap<u32, VecDeque<Order>>,
    asks: HashMap<u32, VecDeque<Order>>,
    index: HashMap<OrderId, (Side, u32)>,
}

impl Default for BitmapBook {
    fn default() -> Self {
        Self::new(TickConfig::default())
    }
}

impl BitmapBook {
    /// A book over the given tick range.
    #[must_use]
    pub fn new(cfg: TickConfig) -> Self {
        let words = (cfg.num_ticks as usize).div_ceil(64);
        BitmapBook {
            cfg,
            bid_bits: vec![0; words],
            ask_bits: vec![0; words],
            bids: HashMap::new(),
            asks: HashMap::new(),
            index: HashMap::new(),
        }
    }

    /// Convert a price to a tick index, or `None` if out of range / not tick-aligned.
    fn price_to_tick(&self, price: Price) -> Option<u32> {
        let offset = (price.0 - self.cfg.min_price) / self.cfg.tick_size;
        if offset.fract() != Decimal::ZERO || offset < Decimal::ZERO {
            return None;
        }
        let tick = offset.to_u32()?;
        (tick < self.cfg.num_ticks).then_some(tick)
    }

    fn tick_to_price(&self, tick: u32) -> Price {
        Price::new(self.cfg.min_price + Decimal::from(tick) * self.cfg.tick_size)
    }

    fn bits(&self, side: Side) -> &[u64] {
        match side {
            Side::Bid => &self.bid_bits,
            Side::Ask => &self.ask_bits,
        }
    }

    fn levels(&self, side: Side) -> &HashMap<u32, VecDeque<Order>> {
        match side {
            Side::Bid => &self.bids,
            Side::Ask => &self.asks,
        }
    }

    fn set_bit(bits: &mut [u64], tick: u32) {
        bits[tick as usize / 64] |= 1u64 << (tick % 64);
    }

    fn clear_bit(bits: &mut [u64], tick: u32) {
        bits[tick as usize / 64] &= !(1u64 << (tick % 64));
    }

    /// Lowest set tick (best ask).
    fn lowest_set(bits: &[u64]) -> Option<u32> {
        for (w, &word) in bits.iter().enumerate() {
            if word != 0 {
                return Some((w * 64) as u32 + word.trailing_zeros());
            }
        }
        None
    }

    /// Highest set tick (best bid).
    fn highest_set(bits: &[u64]) -> Option<u32> {
        for (w, &word) in bits.iter().enumerate().rev() {
            if word != 0 {
                return Some((w * 64) as u32 + (63 - word.leading_zeros()));
            }
        }
        None
    }
}

impl OrderBook for BitmapBook {
    fn insert(&mut self, order: Order) {
        let Some(tick) = self.price_to_tick(order.price) else {
            // Out of the configured tick range: the venue must size its book to its market.
            // Dropping silently would lose orders, so we panic in debug and no-op in release.
            debug_assert!(false, "price {} outside bitmap tick range", order.price);
            return;
        };
        self.index.insert(order.id, (order.side, tick));
        let (bits, levels) = match order.side {
            Side::Bid => (&mut self.bid_bits, &mut self.bids),
            Side::Ask => (&mut self.ask_bits, &mut self.asks),
        };
        Self::set_bit(bits, tick);
        levels.entry(tick).or_default().push_back(order);
    }

    fn best_price(&self, side: Side) -> Option<Price> {
        let tick = match side {
            Side::Bid => Self::highest_set(self.bits(side)),
            Side::Ask => Self::lowest_set(self.bits(side)),
        }?;
        Some(self.tick_to_price(tick))
    }

    fn front_at(&self, side: Side, price: Price) -> Option<(OrderId, Qty)> {
        let tick = self.price_to_tick(price)?;
        self.levels(side)
            .get(&tick)
            .and_then(|l| l.front())
            .map(|o| (o.id, o.qty))
    }

    fn level_at(&self, side: Side, price: Price) -> Vec<(OrderId, Qty)> {
        let Some(tick) = self.price_to_tick(price) else {
            return Vec::new();
        };
        self.levels(side)
            .get(&tick)
            .map(|l| l.iter().map(|o| (o.id, o.qty)).collect())
            .unwrap_or_default()
    }

    fn reduce(&mut self, id: OrderId, qty: Qty) -> bool {
        let Some(&(side, tick)) = self.index.get(&id) else {
            return false;
        };
        let (bits, levels) = match side {
            Side::Bid => (&mut self.bid_bits, &mut self.bids),
            Side::Ask => (&mut self.ask_bits, &mut self.asks),
        };
        let Some(level) = levels.get_mut(&tick) else {
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
                levels.remove(&tick);
                Self::clear_bit(bits, tick);
            }
            removed = true;
        }
        if removed {
            self.index.remove(&id);
        }
        true
    }

    fn get(&self, id: OrderId) -> Option<&Order> {
        let &(side, tick) = self.index.get(&id)?;
        self.levels(side).get(&tick)?.iter().find(|o| o.id == id)
    }

    fn len(&self) -> usize {
        self.index.len()
    }

    fn depth(&self, side: Side, max_levels: usize) -> Vec<(Price, Qty)> {
        let mut out = Vec::new();
        // Walk ticks best-first using the occupancy bitmap.
        let ticks: Vec<u32> = {
            let raw = self.bits(side);
            let mut v: Vec<u32> = (0..self.cfg.num_ticks)
                .filter(|&t| raw[t as usize / 64] & (1u64 << (t % 64)) != 0)
                .collect();
            if side == Side::Bid {
                v.reverse();
            }
            v
        };
        for tick in ticks.into_iter().take(max_levels) {
            if let Some(level) = self.levels(side).get(&tick) {
                let total = level.iter().fold(Qty::ZERO, |acc, o| acc + o.qty);
                out.push((self.tick_to_price(tick), total));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn cfg() -> TickConfig {
        TickConfig {
            min_price: dec!(0),
            tick_size: dec!(1),
            num_ticks: 256,
        }
    }

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
    fn best_price_uses_bitmap_extremes() {
        let mut b = BitmapBook::new(cfg());
        b.insert(ord(1, Side::Bid, 100, 5, 0));
        b.insert(ord(2, Side::Bid, 130, 5, 1)); // spans into the 3rd 64-bit word
        b.insert(ord(3, Side::Ask, 200, 5, 2));
        b.insert(ord(4, Side::Ask, 150, 5, 3));
        assert_eq!(b.best_price(Side::Bid), Some(Price::new(dec!(130))));
        assert_eq!(b.best_price(Side::Ask), Some(Price::new(dec!(150))));
    }

    #[test]
    fn reduce_clears_bit_when_level_empties() {
        let mut b = BitmapBook::new(cfg());
        b.insert(ord(1, Side::Ask, 150, 5, 0));
        assert!(b.reduce(OrderId(1), Qty::new(dec!(5))));
        assert_eq!(b.best_price(Side::Ask), None);
        assert!(b.is_empty());
    }

    #[test]
    fn depth_walks_bitmap_best_first() {
        let mut b = BitmapBook::new(cfg());
        b.insert(ord(1, Side::Bid, 100, 5, 0));
        b.insert(ord(2, Side::Bid, 100, 2, 1));
        b.insert(ord(3, Side::Bid, 99, 4, 2));
        assert_eq!(
            b.depth(Side::Bid, 10),
            vec![
                (Price::new(dec!(100)), Qty::new(dec!(7))),
                (Price::new(dec!(99)), Qty::new(dec!(4))),
            ]
        );
    }

    #[test]
    fn out_of_range_price_is_rejected_in_release() {
        // num_ticks=256 so tick 999 is out of range; price_to_tick returns None.
        let b = BitmapBook::new(cfg());
        assert_eq!(b.price_to_tick(Price::new(dec!(999))), None);
        assert_eq!(b.price_to_tick(Price::new(dec!(-1))), None);
    }
}
