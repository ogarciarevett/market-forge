//! Core domain value types for a matching engine.
//!
//! Money and size are represented with [`rust_decimal::Decimal`] — never `f64` — so matching
//! is deterministic and free of binary floating-point rounding drift.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Side of an order: a bid buys, an ask sells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl Side {
    /// The opposite side — the side an incoming order matches against.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}

/// A monotonically increasing logical timestamp assigned by the engine.
///
/// Using a logical sequence (not wall-clock time) keeps matching deterministic and tests
/// reproducible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

/// A unique order identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OrderId(pub u64);

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// A price level. Ordered numerically; higher is "better" for a bid, lower for an ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Price(pub Decimal);

impl Price {
    #[must_use]
    pub const fn new(value: Decimal) -> Self {
        Price(value)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An order quantity (size). Always non-negative in a well-formed book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Qty(pub Decimal);

impl Qty {
    /// The zero quantity.
    pub const ZERO: Qty = Qty(Decimal::ZERO);

    #[must_use]
    pub const fn new(value: Decimal) -> Self {
        Qty(value)
    }

    /// `true` if this quantity is greater than zero.
    #[must_use]
    pub fn is_positive(self) -> bool {
        self.0 > Decimal::ZERO
    }

    /// The smaller of two quantities — the fillable size when matching two orders.
    #[must_use]
    pub fn min(self, other: Qty) -> Qty {
        if self.0 <= other.0 {
            self
        } else {
            other
        }
    }
}

impl std::ops::Sub for Qty {
    type Output = Qty;
    fn sub(self, rhs: Qty) -> Qty {
        Qty(self.0 - rhs.0)
    }
}

impl std::ops::Add for Qty {
    type Output = Qty;
    fn add(self, rhs: Qty) -> Qty {
        Qty(self.0 + rhs.0)
    }
}

impl fmt::Display for Qty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A resting or incoming order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub price: Price,
    pub qty: Qty,
    pub ts: Timestamp,
}

impl Order {
    #[must_use]
    pub const fn new(id: OrderId, side: Side, price: Price, qty: Qty, ts: Timestamp) -> Self {
        Order {
            id,
            side,
            price,
            qty,
            ts,
        }
    }

    /// Does this incoming order's limit price cross a resting order at `resting`?
    ///
    /// A bid crosses an ask priced at or below its limit; an ask crosses a bid priced at or
    /// above its limit.
    #[must_use]
    pub fn crosses(&self, resting: Price) -> bool {
        match self.side {
            Side::Bid => self.price >= resting,
            Side::Ask => self.price <= resting,
        }
    }
}

/// A trade produced when an incoming (taker) order matches a resting (maker) order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub taker: OrderId,
    pub maker: OrderId,
    pub price: Price,
    pub qty: Qty,
    pub ts: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn side_opposite_is_involutive() {
        assert_eq!(Side::Bid.opposite(), Side::Ask);
        assert_eq!(Side::Ask.opposite(), Side::Bid);
        assert_eq!(Side::Bid.opposite().opposite(), Side::Bid);
    }

    #[test]
    fn qty_min_and_sub() {
        let a = Qty::new(dec!(3));
        let b = Qty::new(dec!(5));
        assert_eq!(a.min(b), a);
        assert_eq!(b - a, Qty::new(dec!(2)));
        assert!(a.is_positive());
        assert!(!Qty::ZERO.is_positive());
    }

    #[test]
    fn bid_crosses_lower_ask() {
        let bid = Order::new(
            OrderId(1),
            Side::Bid,
            Price::new(dec!(100)),
            Qty::new(dec!(1)),
            Timestamp(0),
        );
        assert!(bid.crosses(Price::new(dec!(99))));
        assert!(bid.crosses(Price::new(dec!(100))));
        assert!(!bid.crosses(Price::new(dec!(101))));
    }

    #[test]
    fn ask_crosses_higher_bid() {
        let ask = Order::new(
            OrderId(2),
            Side::Ask,
            Price::new(dec!(100)),
            Qty::new(dec!(1)),
            Timestamp(0),
        );
        assert!(ask.crosses(Price::new(dec!(101))));
        assert!(ask.crosses(Price::new(dec!(100))));
        assert!(!ask.crosses(Price::new(dec!(99))));
    }
}
