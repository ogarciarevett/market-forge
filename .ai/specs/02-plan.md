# 02 — Build plan (task breakdown)

- Status: active
- Derives from: [`01-spec.md`](./01-spec.md)

## Priority
Must-have core: S0–S6 (workspace + SDK + wizard + codegen + a generated venue that
build/test/benches). Documentation half: S7 (all catalogs). Polish: S8–S9 (TUI, web).
Each slice leaves the tree green (`cargo check/clippy/fmt/test`) before the next.

---

### S0 — Native baseline + workspace (CLA-140)  ⟸ start here
Files: `Cargo.toml` (workspace), `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE.md`,
`CONTRIBUTING.md`, `README.md`, `rust-toolchain.toml`, `.gitignore` (+ `target/`), `crates/mf-core/`.
- Workspace skeleton; dual MIT/Apache license; attribution to OrderBook-rs; fork/sync policy;
  `mf-core` domain types (Price/Qty/Order/Trade/Side + OrderBook & MatchingEngine traits).
- **Acceptance:** `cargo build` + `cargo test` green on `mf-core`; clippy/fmt clean.

### S1 — Standards + crates inventory (CLA-146)
Files: `docs/standards.md`, `crates/xtask/src/catalog.rs`.
- Mermaid conventions, plain-language style, crates inventory (license/health/audit), lint.
- **Acceptance:** `cargo xtask lint-catalog` runs; `cargo audit` clean.

### S2 — Architecture spike (CLA-136)
Files: `docs/architecture.md`.
- 9 decisions w/ rationale, Mermaid flow + codegen pipeline, inherited-vs-new, combo matrix.
- **Acceptance:** doc present, decisions logged, diagrams render.

### S3 — SDK: matching + book + concurrency (FIFO/ProRata, BTree/Bitmap, single/disruptor)
Files: `crates/mf-matching/`, `crates/mf-book/`, `crates/mf-concurrency/`.
- Implement the MVP-critical primitives behind the core traits, unit + property tested.
- **Acceptance:** golden FIFO tests, qty-conservation proptest, clippy clean.

### S4 — Codegen engine (CLA-136/147)
Files: `crates/mf-codegen/`, `templates/`.
- `VenueSpec` + serde validation + compatibility matrix; Tera rendering to a workspace tree.
- **Acceptance:** unit tests render templates; invalid combos rejected with a message.

### S5 — CLI wizard (CLA-147)
Files: `crates/market-forge/` (bin): `clap` `new/list/validate`, `inquire` 3-prompt wizard.
- **Acceptance:** `market-forge new` non-interactive (flags) + interactive paths produce a tree.

### S6 — MVP end-to-end proof (CLA-147)
Files: generated `examples/` snapshot + an integration test that generates+builds+tests a venue.
- **Acceptance:** generated workspace `cargo build --release` + `cargo test` + `cargo bench` green.

### S7 — Algorithm catalogs (CLA-137/138/139/141/142/143/144/145)
Files: `docs/catalog/{matching,books,concurrency,risk,liquidation,perps,prediction,infra}/*.md`.
- ~70 entries, each Mermaid + ≤300-word prose + crate + venue, passing the catalog lint.
- **Acceptance:** lint passes for every entry; counts match (10/8/10/12/8/6/10/12).

### S8 — TUI visualization (CLA-148)
Files: `templates/.../<venue>-tui/` + an SDK `mf-tui` helper.
- **Acceptance:** generated `-tui` binary builds; renders depth/price/tape/latency.

### S9 — Web visualization (CLA-149)
Files: `templates/.../<venue>-web/` (axum + embedded TradingView Lightweight Charts).
- **Acceptance:** generated `-web` binary builds; serves bundle + WebSocket.

---

## Parallel-work notes
- S7 catalogs are file-disjoint per entry → fan out with the `Workflow` tool (one agent per
  category or per entry), then lead runs the catalog lint over the merged set.
- S3 SDK crates are file-disjoint (matching vs book vs concurrency) but share `mf-core` traits;
  build `mf-core` (S0) first, then they can go in parallel.
- S8/S9 are template-only and independent of each other.
- Do NOT parallelize S4/S5/S6 — tightly coupled codegen path; one focused agent.
