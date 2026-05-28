//! [`ProRataMatcher`] — pro-rata matching: an incoming order fills against the best opposite
//! price by allocating its size across *every* resting order at that level in proportion to
//! each order's size (not time priority). Common in futures/options venues to reward large
//! resting liquidity.
//!
//! Allocation is deterministic: a cumulative-target method that gives the final order the
//! exact remainder, so the per-order fills always sum to what was actually matched (quantity
//! is conserved — no dust created or destroyed).

use mf_core::{MatchingEngine, Order, OrderBook, OrderId, Qty, Timestamp, Trade};
use rust_decimal::Decimal;

/// Decimal places to which intermediate pro-rata targets are rounded.
const ALLOC_SCALE: u32 = 18;

/// A pro-rata matching engine over any [`OrderBook`].
#[derive(Debug)]
pub struct ProRataMatcher<B: OrderBook> {
    book: B,
    seq: u64,
}

impl<B: OrderBook> ProRataMatcher<B> {
    /// Wrap a book in a pro-rata matcher.
    pub fn new(book: B) -> Self {
        ProRataMatcher { book, seq: 0 }
    }

    /// Borrow the underlying book (for snapshots / visualization).
    pub fn book(&self) -> &B {
        &self.book
    }

    fn next_ts(&mut self) -> Timestamp {
        self.seq += 1;
        Timestamp(self.seq)
    }
}

/// Allocate `fill` across `level` (each `(id, resting_qty)`) in proportion to resting size.
/// Returns the per-order allocation aligned to `level`. The allocations sum to `fill` (modulo
/// the final-remainder guarantee) and never exceed any order's resting quantity.
fn allocate(level: &[(OrderId, Qty)], fill: Decimal) -> Vec<Decimal> {
    let total: Decimal = level.iter().map(|(_, q)| q.0).sum();
    let mut out = Vec::with_capacity(level.len());
    if total.is_zero() || fill.is_zero() {
        out.resize(level.len(), Decimal::ZERO);
        return out;
    }
    let mut allocated = Decimal::ZERO;
    let mut cum_q = Decimal::ZERO;
    let last = level.len() - 1;
    for (i, (_, q)) in level.iter().enumerate() {
        cum_q += q.0;
        let alloc = if i == last {
            // Final order takes the remainder, clamped to its size — keeps the sum exact.
            (fill - allocated).min(q.0).max(Decimal::ZERO)
        } else {
            let target = (fill * cum_q / total).round_dp(ALLOC_SCALE);
            (target - allocated).clamp(Decimal::ZERO, q.0)
        };
        allocated += alloc;
        out.push(alloc);
    }
    out
}

impl<B: OrderBook> MatchingEngine for ProRataMatcher<B> {
    fn submit(&mut self, mut taker: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let opp = taker.side.opposite();

        while taker.qty.is_positive() {
            let Some(best) = self.book.best_price(opp) else {
                break;
            };
            if !taker.crosses(best) {
                break;
            }
            let level = self.book.level_at(opp, best);
            if level.is_empty() {
                break;
            }
            let total: Decimal = level.iter().map(|(_, q)| q.0).sum();
            let fill = taker.qty.0.min(total);
            let allocs = allocate(&level, fill);

            let mut applied = Decimal::ZERO;
            for ((id, _), alloc) in level.iter().zip(allocs.iter()) {
                if alloc.is_zero() {
                    continue;
                }
                self.book.reduce(*id, Qty(*alloc));
                let ts = self.next_ts();
                trades.push(Trade {
                    taker: taker.id,
                    maker: *id,
                    price: best,
                    qty: Qty(*alloc),
                    ts,
                });
                applied += *alloc;
            }

            if applied.is_zero() {
                break; // no progress (dust) — avoid spinning
            }
            taker.qty = Qty(taker.qty.0 - applied);
        }

        if taker.qty.is_positive() {
            self.book.insert(taker);
        }
        trades
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_book::BTreeBook;
    use mf_core::{Order, OrderId, Price, Side};
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
    fn allocates_in_proportion_to_resting_size() {
        let mut m = ProRataMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 10)); // 25% of the level
        m.submit(ord(2, Side::Ask, 100, 30)); // 75% of the level
        let trades = m.submit(ord(3, Side::Bid, 100, 20));
        assert_eq!(trades.len(), 2);
        let a = trades.iter().find(|t| t.maker == OrderId(1)).unwrap().qty;
        let b = trades.iter().find(|t| t.maker == OrderId(2)).unwrap().qty;
        assert_eq!(a, Qty::new(dec!(5))); // 25% of 20
        assert_eq!(b, Qty::new(dec!(15))); // 75% of 20
        assert_eq!((a + b), Qty::new(dec!(20))); // quantity conserved
    }

    #[test]
    fn fills_sum_to_taker_size_with_uneven_split() {
        // 3-way split of 10 across 1:1:1 → must still total exactly 10.
        let mut m = ProRataMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 1));
        m.submit(ord(2, Side::Ask, 100, 1));
        m.submit(ord(3, Side::Ask, 100, 1));
        let trades = m.submit(ord(4, Side::Bid, 100, 3));
        let total: Decimal = trades.iter().map(|t| t.qty.0).sum();
        assert_eq!(total, dec!(3));
        assert!(m.book().is_empty());
    }

    #[test]
    fn consumes_level_then_walks_to_next_price() {
        let mut m = ProRataMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 2));
        m.submit(ord(2, Side::Ask, 101, 5));
        let trades = m.submit(ord(3, Side::Bid, 101, 4));
        let total: Decimal = trades.iter().map(|t| t.qty.0).sum();
        assert_eq!(total, dec!(4)); // 2 @100 + 2 @101
        assert_eq!(
            m.book().get(OrderId(2)).map(|o| o.qty),
            Some(Qty::new(dec!(3)))
        );
    }
}
