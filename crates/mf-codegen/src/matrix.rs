//! The compatibility matrix: which combinations of choices coherently compose.
//!
//! For the MVP's four matching×book combinations every pairing is at least *workable*; the
//! matrix exists so future, richer catalogs (e.g. an LMSR prediction-market maker against a
//! CLOB FIFO matcher) can be rejected with a clear reason before any code is generated.

use crate::error::CodegenError;
use crate::spec::{BookKind, Concurrency, Matching, VenueSpec};

/// How well a combination composes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    /// Idiomatic, recommended pairing.
    Natural,
    /// Works, with a caveat worth knowing.
    Workable(&'static str),
    /// Cannot coherently compose; generation must abort.
    Incoherent(&'static str),
}

/// Evaluate the compatibility of a fully-specified venue.
#[must_use]
pub fn evaluate(spec: &VenueSpec) -> Compatibility {
    match (spec.matching, spec.book, spec.concurrency) {
        // Pro-rata over a bitmap book needs per-order size tracking at each tick: supported,
        // but heavier than the BTree book's natural level queues.
        (Matching::ProRata, BookKind::Bitmap, _) => Compatibility::Workable(
            "pro-rata over a bitmap book tracks per-order size at each tick — works, but the \
             BTreeMap book is the lighter pairing",
        ),
        // A bitmap book is bounded-tick; a Disruptor front-end doesn't change that, but the
        // venue must size its tick range to its market.
        (_, BookKind::Bitmap, Concurrency::Disruptor) => Compatibility::Workable(
            "bitmap book is bounded-tick; ensure the configured tick range covers your market",
        ),
        _ => Compatibility::Natural,
    }
}

/// Validate that a venue's combination can be generated. Errors only on an incoherent combo.
pub fn check(spec: &VenueSpec) -> Result<(), CodegenError> {
    match evaluate(spec) {
        Compatibility::Incoherent(why) => Err(CodegenError::Incoherent(why.to_string())),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(m: Matching, b: BookKind, c: Concurrency) -> VenueSpec {
        VenueSpec {
            name: "x".into(),
            matching: m,
            book: b,
            concurrency: c,
        }
    }

    #[test]
    fn fifo_btree_single_is_natural() {
        let s = spec(
            Matching::Fifo,
            BookKind::BTreeMap,
            Concurrency::SingleThread,
        );
        assert_eq!(evaluate(&s), Compatibility::Natural);
        assert!(check(&s).is_ok());
    }

    #[test]
    fn prorata_bitmap_is_workable_not_rejected() {
        let s = spec(
            Matching::ProRata,
            BookKind::Bitmap,
            Concurrency::SingleThread,
        );
        assert!(matches!(evaluate(&s), Compatibility::Workable(_)));
        assert!(check(&s).is_ok());
    }
}
