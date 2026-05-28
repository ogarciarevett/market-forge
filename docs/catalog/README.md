# Market Forge — Algorithm Catalog

Generated reference of every algorithm the wizard can offer. Each entry follows
[`../standards.md`](../standards.md): a Mermaid diagram + a ≤300-word plain-language
explanation (what it is · when to pick · when not · real venue · recommended crate).

## Matching engines (CLA-137) — 10

- [Dutch Auction](matching/dutch-auction.md)
- [Iceberg / Reserve Order Matching](matching/iceberg-reserve.md)
- [Pegged Orders / Mid-Point Match](matching/pegged-midpoint.md)
- [Price-Size-Time (Size-Pro-Rata Hybrid)](matching/price-size-time.md)
- [Price-Time Priority (FIFO)](matching/price-time-fifo.md)
- [Pro-Rata with TOP](matching/pro-rata-top.md)
- [Pro-Rata Matching](matching/pro-rata.md)
- [Self-Trade Prevention (STP)](matching/self-trade-prevention.md)
- [Threshold Pro-Rata](matching/threshold-pro-rata.md)
- [Auction Matching (Uniform Price Auction)](matching/uniform-price-auction.md)

## Order-book data structures (CLA-138) — 9

- [Adaptive Radix Tree (ART) Book](books/adaptive-radix-tree.md)
- [Array-Based Price Ladder](books/array-price-ladder.md)
- [Bitmap / Bit-Indexed Book](books/bitmap.md)
- [BTreeMap Book](books/btreemap.md)
- [Hash + Doubly-Linked List per Level](books/hash-linked-list.md)
- [Order ID to Location Index](books/order-id-location-index.md)
- [Skip List Book](books/skip-list.md)
- [Slab Allocator / Order Pool](books/slab-allocator.md)
- [Van Emde Boas / Y-Fast Trie Book](books/van-emde-boas.md)

## Concurrency & throughput (CLA-139) — 10

- [Cache-Line Padding (False-Sharing Prevention)](concurrency/cache-line-padding.md)
- [LMAX Disruptor](concurrency/lmax-disruptor.md)
- [MPSC with Back-Pressure](concurrency/mpsc-backpressure.md)
- [NUMA-Aware Sharding](concurrency/numa-aware-sharding.md)
- [Read-Copy-Update (RCU)](concurrency/rcu.md)
- [SeqLock](concurrency/seqlock.md)
- [Sharding by Symbol](concurrency/sharding-by-symbol.md)
- [Single-Writer Principle](concurrency/single-writer-principle.md)
- [SPSC Ring Buffer](concurrency/spsc-ring-buffer.md)
- [Wait-Free vs Lock-Free Queues](concurrency/waitfree-lockfree-queues.md)

## Risk — margin & PnL (CLA-141) — 13

- [Binomial / Trinomial Trees](risk/binomial-trinomial-trees.md)
- [Black-Scholes / Black-76 / Bachelier](risk/black-scholes.md)
- [Expected Shortfall (CVaR/TVaR)](risk/expected-shortfall-cvar.md)
- [Greeks (Delta/Gamma/Vega/Theta/Rho)](risk/greeks.md)
- [Isolated Margin](risk/isolated-margin.md)
- [Mark-to-Market vs Mark-to-Index](risk/mark-to-market-vs-index.md)
- [Portfolio / Cross-Margin](risk/portfolio-cross-margin.md)
- [Real-Time Incremental PnL](risk/realtime-pnl-incremental.md)
- [SPAN 2 / Expected-Shortfall Margin](risk/span-2-expected-shortfall.md)
- [SPAN (Standard Portfolio Analysis of Risk)](risk/span.md)
- [Stress Testing](risk/stress-testing.md)
- [Tiered / Bracketed Margin](risk/tiered-margin.md)
- [Value at Risk (VaR)](risk/value-at-risk.md)

## Liquidation & solvency (CLA-142) — 8

- [Auto-Deleveraging (ADL)](liquidation/auto-deleveraging-adl.md)
- [Backstop Liquidity Provider (BLP) Queues](liquidation/backstop-liquidity-provider.md)
- [Bankruptcy Price / Insurance-Fund Trigger](liquidation/bankruptcy-price-insurance-trigger.md)
- [Liquidation Cascade Prevention](liquidation/cascade-prevention.md)
- [Insurance Fund Management](liquidation/insurance-fund-management.md)
- [Liquidation Engine](liquidation/liquidation-engine.md)
- [Partial Liquidation](liquidation/partial-liquidation.md)
- [Socialized Loss](liquidation/socialized-loss.md)

## Perps-specific mechanics (CLA-143) — 6

- [Funding Rate](perps/funding-rate.md)
- [Impact Bid/Ask Price](perps/impact-bid-ask.md)
- [Index Price Aggregation](perps/index-price-aggregation.md)
- [Maker/Taker Fee Tiers](perps/maker-taker-fee-tiers.md)
- [Mark Price](perps/mark-price.md)
- [Position Netting vs Hedge Mode](perps/position-netting-hedge-mode.md)

## Prediction-market makers (CLA-144) — 10

- [Bayesian Market Maker](prediction/bayesian-market-maker.md)
- [CFMM for Binary Outcomes](prediction/cfmm-binary-outcomes.md)
- [Combinatorial Prediction Markets](prediction/combinatorial-prediction-markets.md)
- [Conditional Tokens Framework (CTF)](prediction/conditional-tokens-framework.md)
- [Dynamic Pari-Mutuel Market (DPM)](prediction/dynamic-pari-mutuel.md)
- [Liquidity Bootstrapping / Maniac MM](prediction/liquidity-bootstrapping.md)
- [LMSR (Logarithmic Market Scoring Rule)](prediction/lmsr.md)
- [LS-LMSR (Liquidity-Sensitive LMSR)](prediction/ls-lmsr.md)
- [Oracle Resolution](prediction/oracle-resolution.md)
- [Polymarket-Style CLOB](prediction/polymarket-clob.md)

## Cross-cutting infrastructure (CLA-145) — 12

- [Aeron / Chronicle IPC](infra/aeron-chronicle.md)
- [Branch-Prediction Hints](infra/branch-prediction-hints.md)
- [CPU Pinning (isolcps / nohz_full)](infra/cpu-pinning.md)
- [Deterministic Replay](infra/deterministic-replay.md)
- [Event Sourcing + CQRS](infra/event-sourcing-cqrs.md)
- [FIX / ITCH / OUCH / SBE Protocols](infra/fix-itch-ouch-sbe.md)
- [Fixed-Point Decimals](infra/fixed-point-decimals.md)
- [Huge Pages / mlock](infra/huge-pages-mlock.md)
- [Kernel-Bypass Networking](infra/kernel-bypass-networking.md)
- [SIMD Batch Compute](infra/simd.md)
- [Write-Ahead Log (WAL)](infra/write-ahead-log.md)
- [Zero-Copy Serialization](infra/zero-copy-serialization.md)

