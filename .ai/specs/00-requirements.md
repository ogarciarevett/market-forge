# 00 — Requirements (verbatim capture)

> Source: Linear project **Market Forge — interactive matching-engine generator + SDK**
> (claw-village / CLA), issues CLA-136 .. CLA-149. Captured 2026-05-29.

## Problem
Designing a production-grade matching engine forces dozens of orthogonal, interacting
decisions (matching algorithm, book data structure, concurrency model, risk/margin model,
liquidation, perps mechanics, prediction-market makers, cross-cutting infra). There is no
tool that lets an engineer *choose* those design points and get a working, idiomatic Rust
venue scaffold with the trade-offs documented. Market Forge is that tool: an interactive CLI
that asks about the design and generates a tailored, compiling, benchable matching engine,
plus an SDK of à-la-carte crates for users who skip the wizard. Audience: Rust engineers and
quant-curious developers.

## Functional requirements
- **CLI generator** — `market-forge new <venue>` runs an interactive wizard and emits a
  self-contained Cargo workspace. (CLA-136, CLA-147)
- **MVP slice (CLA-147)** — 3 prompts: matching (FIFO | Pro-Rata) · book (BTreeMap | Bitmap) ·
  concurrency (single-thread | LMAX/DMAX Disruptor). Generated workspace `cargo build
  --release`s, `cargo test`s (golden FIFO behavior), `cargo bench`es (Criterion), ships a
  README explaining what/why.
- **SDK** — matching/book/concurrency primitives available as crates so users can
  `cargo add` directly without the CLI. (CLA-136)
- **Native baseline + attribution (CLA-140)** — re-implement the OrderBook-rs design lineage
  natively (LMAX/DMAX Disruptor as default concurrency template), credit Joaquín Béjar García
  in `NOTICE.md`/README, document upstream-sync policy, `CONTRIBUTING.md` "this is a fork"
  section. License audit → MIT/Apache-2.0 dual.
- **Standards (CLA-146)** — `docs/standards.md`: Mermaid diagram conventions, ≤300-word
  plain-language style, Rust crates inventory (license + health + `cargo audit` clean), and a
  lint script that flags any algorithm doc missing a required section.
- **Algorithm catalogs** — each entry = Mermaid diagram + ≤300-word plain-language
  explanation + recommended crate + real-venue example:
  - Matching engines × 10 (CLA-137)
  - Order-book data structures × 8 (CLA-138)
  - Concurrency & throughput × 10 (CLA-139)
  - Risk manager / margin & PnL × 12 (CLA-141)
  - Liquidation & solvency × 8 (CLA-142)
  - Perps-specific mechanics × 6 (CLA-143)
  - Prediction-market makers × 10 (CLA-144)
  - Cross-cutting infrastructure × 12 (CLA-145)
- **Combination matrix** — rules engine that rejects incoherent combinations (e.g. LMSR+FIFO)
  with a colored compatibility graph. (CLA-136, CLA-146)
- **Visualization v0.1 (CLA-148)** — generated venue ships an optional `<venue>-tui` binary
  (`ratatui`): live depth + price + trade tape + latency at 30–60 fps.
- **Visualization v0.2 (CLA-149)** — optional `<venue>-web` binary: `axum` WebSocket + a
  self-contained HTML/JS bundle using TradingView Lightweight Charts (no npm install).

## Non-functional requirements / constraints
- Language LOCKED to **Rust** (stable 1.94, edition 2021).
- Deterministic arithmetic for money (`rust_decimal`); never `f64` for price/qty.
- Lock-free matching hot path; no required async runtime on it.
- No GPL/LGPL dependencies (would constrain downstream commercial use of generated code).
- Every generated workspace MUST build + test + bench. A generator emitting broken code fails.
- `cargo audit` clean; reject crates with open RUSTSEC advisories.
- Docs render on GitHub/Linear (Mermaid inline, ≤~50 lines per diagram).

## Out of scope / non-goals
- Remote GitHub fork, pushing, PRs, deploys, and contacting Joaquín — human-only (hard rules).
- MVP excludes: risk model, liquidation, perps, prediction-market modes, >3 wizard prompts,
  third-party plugin discovery (deferred past MVP).
- Real exchange connectivity / live market data.

## Acceptance criteria (seed)
- `market-forge new my-clob` → generated workspace builds, tests (golden FIFO), and benches.
- `docs/architecture.md` (9 decisions) approved before catalog implementation.
- `docs/standards.md` present + lint script enforces doc sections.
- All catalog entries present and standard-compliant.
- Generated venue `-tui` renders live; `-web` serves charts over WebSocket.
- Expanded per-requirement in `99-acceptance.md`.
