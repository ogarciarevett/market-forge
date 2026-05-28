//! [`FifoMatcher`] — classic price-time (FIFO) matching: an incoming order fills against the
//! best opposite price, taking resting orders in strict time priority (oldest first).

use mf_core::{MatchingEngine, Order, OrderBook, Timestamp, Trade};

/// A price-time matching engine over any [`OrderBook`].
#[derive(Debug)]
pub struct FifoMatcher<B: OrderBook> {
    book: B,
    seq: u64,
}

impl<B: OrderBook> FifoMatcher<B> {
    /// Wrap a book in a FIFO matcher.
    pub fn new(book: B) -> Self {
        FifoMatcher { book, seq: 0 }
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

impl<B: OrderBook> MatchingEngine for FifoMatcher<B> {
    fn submit(&mut self, mut taker: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let opp = taker.side.opposite();

        while taker.qty.is_positive() {
            let Some(best) = self.book.best_price(opp) else {
                break; // no opposite liquidity
            };
            if !taker.crosses(best) {
                break; // best opposite price no longer crosses our limit
            }
            let Some((maker_id, maker_qty)) = self.book.front_at(opp, best) else {
                break;
            };
            let fill = taker.qty.min(maker_qty);
            let reduced = self.book.reduce(maker_id, fill);
            debug_assert!(
                reduced,
                "reduce on a maker we just read from the book failed"
            );
            let ts = self.next_ts();
            trades.push(Trade {
                taker: taker.id,
                maker: maker_id,
                price: best, // resting (maker) price — price-improvement goes to the taker
                qty: fill,
                ts,
            });
            taker.qty = taker.qty - fill;
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
    use mf_core::{Order, OrderId, Price, Qty, Side};
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
    fn resting_order_does_not_cross_empty_book() {
        let mut m = FifoMatcher::new(BTreeBook::new());
        let trades = m.submit(ord(1, Side::Bid, 100, 5));
        assert!(trades.is_empty());
        assert_eq!(m.book().best_price(Side::Bid), Some(Price::new(dec!(100))));
    }

    #[test]
    fn full_fill_at_resting_price() {
        let mut m = FifoMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 5));
        let trades = m.submit(ord(2, Side::Bid, 101, 5));
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, Price::new(dec!(100))); // taker gets price improvement
        assert_eq!(trades[0].qty, Qty::new(dec!(5)));
        assert!(m.book().is_empty());
    }

    #[test]
    fn time_priority_oldest_maker_fills_first() {
        let mut m = FifoMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 3)); // older
        m.submit(ord(2, Side::Ask, 100, 3)); // newer
        let trades = m.submit(ord(3, Side::Bid, 100, 4));
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].maker, OrderId(1)); // oldest first
        assert_eq!(trades[0].qty, Qty::new(dec!(3)));
        assert_eq!(trades[1].maker, OrderId(2));
        assert_eq!(trades[1].qty, Qty::new(dec!(1)));
        // 2 of order #2's 3 remain resting.
        assert_eq!(
            m.book().get(OrderId(2)).map(|o| o.qty),
            Some(Qty::new(dec!(2)))
        );
    }

    #[test]
    fn partial_taker_rests_remainder() {
        let mut m = FifoMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 2));
        let trades = m.submit(ord(2, Side::Bid, 100, 5));
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, Qty::new(dec!(2)));
        // 3 remainder rests on the bid side.
        assert_eq!(m.book().best_price(Side::Bid), Some(Price::new(dec!(100))));
        assert_eq!(
            m.book().get(OrderId(2)).map(|o| o.qty),
            Some(Qty::new(dec!(3)))
        );
    }

    #[test]
    fn walks_multiple_price_levels() {
        let mut m = FifoMatcher::new(BTreeBook::new());
        m.submit(ord(1, Side::Ask, 100, 2));
        m.submit(ord(2, Side::Ask, 101, 2));
        let trades = m.submit(ord(3, Side::Bid, 101, 4));
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, Price::new(dec!(100)));
        assert_eq!(trades[1].price, Price::new(dec!(101)));
        assert!(m.book().is_empty());
    }
}
