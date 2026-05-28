# 98 — Nice-to-haves & deferred backlog

## A. Applied in the finish pass
- Bitmap `depth` best-first word-walk (was O(num_ticks)) — `mf-book/src/bitmap.rs`.
- `reduce` `#[must_use]` + matcher asserts; `Qty::Sub` underflow guard — `mf-core`, matchers.
- `BitmapBook::insert` fails loudly on out-of-range price (no silent drop) — `mf-book`.
- Exhaustive wizard matches; dropped needless `PathBuf` clones — `market-forge`.
- Path-traversal/injection name-rejection regression test — `mf-codegen`.

## B. Deferred (documented, not built)

### Throughput / scale
- Zero-alloc match path: `OrderBook::level_apply` visitor so pro-rata allocates against the
  book without materializing a `Vec` per level; reuse scratch `Vec`s in `ProRataMatcher`.
  Cost: M; gate on a `dhat` allocation bench first.
- `submit` returning `Vec<Trade>`: offer a `&mut Vec<Trade>` / callback sink for zero-alloc
  trade emission. Cost: S.
- Pro-rata `reduce` is O(k) per reduced order (O(k²) to clear a deep level): switch to a
  slab + `id→slot` map if a bench shows deep levels matter. Cost: M.
- Real cross-thread Disruptor (consumer on its own core); current runner is single-thread
  publish-then-drain with the same sequencing discipline. Cost: M; needs `loom` tests.

### Correctness / durability
- Promote `OrderBook::insert`/`MatchingEngine::submit` to `Result` so a bounded-book
  overflow is a typed error rather than a panic. Cost: M (trait + all callers + templates).
- Property tests (`proptest`) for book/matcher invariants (qty conservation, no crossed book)
  beyond the current example-based unit tests. Cost: S–M.

### Security hardening
- Tera HTML autoescape (register web template under an `.html` name) + CSP/SRI on the CDN
  `lightweight-charts` script — only matters if the generated `-web` is ever network-exposed.
- Validate `--sdk-path` rejects `"`/newline before rendering into `Cargo.toml`. Cost: S.

### Functionality / ops
- The remaining catalog algorithms as actual SDK implementations (only the MVP set —
  FIFO/ProRata, BTree/Bitmap, single/Disruptor — is coded; the other ~70 are documented).
- Plugin discovery for third-party algorithms (deferred past MVP per CLA-147).
- `forge.toml` advanced track to toggle the infra patterns (WAL, event-sourcing, etc.).
- 90-second screencast asset for the MVP (CLA-147 public moment) — human-recorded.
