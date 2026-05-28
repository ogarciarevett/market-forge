//! Integration test: every matching×book×concurrency combination renders a complete,
//! plausible venue tree. (A full `cargo build` of a generated venue is exercised manually /
//! in CI via `market-forge new`; see `.ai/specs/99-acceptance.md`.)

use mf_codegen::{render_venue, BookKind, Concurrency, Matching, VenueSpec};
use std::path::Path;

const MATCHING: [Matching; 2] = [Matching::Fifo, Matching::ProRata];
const BOOKS: [BookKind; 2] = [BookKind::BTreeMap, BookKind::Bitmap];
const CONC: [Concurrency; 2] = [Concurrency::SingleThread, Concurrency::Disruptor];

#[test]
fn every_combination_renders_a_complete_tree() {
    let mut n = 0;
    for m in MATCHING {
        for b in BOOKS {
            for c in CONC {
                let spec = VenueSpec {
                    name: "combo-venue".into(),
                    matching: m,
                    book: b,
                    concurrency: c,
                };
                let tmp = tempfile::tempdir().unwrap();
                let written = render_venue(&spec, tmp.path(), Path::new("/sdk"))
                    .unwrap_or_else(|e| panic!("render failed for {m:?}/{b:?}/{c:?}: {e}"));
                assert_eq!(written.len(), 6, "expected 6 files for {m:?}/{b:?}/{c:?}");

                let lib =
                    std::fs::read_to_string(tmp.path().join("crates/engine/src/lib.rs")).unwrap();
                // The engine wires the exact chosen types.
                let matcher = if m == Matching::Fifo {
                    "FifoMatcher"
                } else {
                    "ProRataMatcher"
                };
                let book = if b == BookKind::BTreeMap {
                    "BTreeBook"
                } else {
                    "BitmapBook"
                };
                let runner = if c == Concurrency::SingleThread {
                    "SingleThreadRunner"
                } else {
                    "DisruptorRunner"
                };
                assert!(lib.contains(matcher), "lib.rs missing {matcher}");
                assert!(lib.contains(book), "lib.rs missing {book}");
                assert!(lib.contains(runner), "lib.rs missing {runner}");
                n += 1;
            }
        }
    }
    assert_eq!(n, 8, "expected all 8 combinations");
}
