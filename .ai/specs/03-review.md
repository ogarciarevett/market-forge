# 03 — Consensus Review

- Date: 2026-05-29
- Method: 3-agent parallel panel (rust-reviewer · security-auditor · performance-reviewer)
  over the SDK + generator. Gate: 2-of-3 APPROVE.
- Traces to: [`02-plan.md`](./02-plan.md)

## Verdicts

| Reviewer | Verdict |
|----------|---------|
| security-auditor | **APPROVE** (0 critical/high; 2 low, defense-in-depth) |
| performance-reviewer | **APPROVE** (no SLO violation; 1 important: bitmap `depth`) |
| rust-reviewer | CHANGES (2 critical, 3 high, 3 medium) |

**Outcome: PASS (2-of-3 APPROVE).** All actionable findings were nonetheless resolved
(Chesterton's Fence: fix real issues, document deliberate tradeoffs).

## Findings & resolutions

| ID | Sev | Finding | Resolution |
|----|-----|---------|------------|
| C1 | CRIT | `BitmapBook::insert` silently dropped out-of-range orders in release (`debug_assert!`+return) | Now `panic!`s with a clear "widen TickConfig" message — bounded book must be sized to its market; no silent liquidity loss. `bitmap.rs` |
| C2 | CRIT | `OrderBook::reduce` return value dropped on the match path → could create taker qty | `reduce` marked `#[must_use]`; both matchers capture it under `debug_assert!`. `book.rs`, `fifo.rs`, `prorata.rs` |
| H1 | HIGH | Pro-rata rounding could leave sub-ulp dust on non-terminating splits | Confirmed conservation holds by the final-remainder design; added `fractional_split_conserves_quantity_exactly` (1 split 3 ways → sums to exactly 1). `prorata.rs` |
| H2 | HIGH | `Qty::Sub` unchecked → negative qty could rest | Added underflow `debug_assert!`. `types.rs` |
| H3 | HIGH | Wizard wildcard `_` match would absorb future enum variants as the default | Exhaustive matches with `unreachable!()` on the impossible branch. `wizard.rs` |
| M1 | MED | `BitmapBook::depth` scanned all `num_ticks` (≤100k) every call — fires 30–60 fps in the TUI | Rewrote as a best-first word-walk that stops at `max_levels`: O(max_levels + words). Added `depth_best_first_walk_spans_multiple_words`. `bitmap.rs` |
| M2 | MED | Needless `.clone()` on `Option<PathBuf>` in `run_new` | Moved instead of cloned. `main.rs` |
| M3 | MED | Disruptor `drain` silently advanced the cursor on a `None` slot | `expect`s the published-before-consumed invariant. `disruptor.rs` |
| L1 | LOW | `allocate` not inlined on the hot path | `#[inline]`. `prorata.rs` |
| L2 | LOW | `len`/`is_empty` not `#[must_use]` | Added. `book.rs` |
| Sec-L | LOW | Tera HTML template not autoescaped (defense-in-depth) | Locked the invariant with `rejects_path_traversal_and_injection_names` proving the `[a-z0-9-]` name charset blocks `/ .. " < >`. `render.rs` |

## Documented acceptable tradeoffs (reference impl, not "must fix")
- `level_at`/`depth` return `Vec` (allocation per matched level): acceptable for the reference
  SDK; perf-reviewer's scratch-`Vec` optimization noted in `98-nice-to-haves.md`.
- `Decimal` arithmetic cost: deliberate, spec-mandated (no `f64` for money).
- `reduce` is O(k) in level depth for pro-rata (O(1) common-path for FIFO): noted, bench-gated.
- `Vec<Trade>` return per `submit`: reference API shape; sink/callback is a nice-to-have.

Post-fix state: `cargo test --workspace` 32 green · `cargo clippy --workspace --all-targets
-D warnings` clean · `cargo fmt --check` clean · generated venues still build/test/bench.
