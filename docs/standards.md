# Market Forge — Documentation & Crate Standards

> Locked by CLA-146. Every algorithm-catalog entry under `docs/catalog/` MUST follow this.
> `scripts/lint-catalog.sh` enforces the structural rules below.

## 1. Diagram conventions

- **Mermaid is the default.** It renders inline on GitHub, Linear, and most doc sites.
- **Per-algorithm diagram** = one ```` ```mermaid ```` block, either a `sequenceDiagram` or a
  `flowchart`. Pick ONE diagram kind per category and use it consistently across that
  category's entries.
- **Combination-matrix diagrams** = `graph` with edges colored by compatibility:
  `green = natural`, `yellow = workable`, `red = incoherent`.
- **Theme** — use Mermaid's default theme; do not customize colors per diagram (except the
  matrix edge colors above).
- **Naming** — node ids `lower_snake_case`; display labels Title Case; all labels in English.
- **Length cap** — no diagram exceeds ~50 lines of Mermaid source (one 1080p screen).

## 2. Plain-language explanation style

Each entry's prose is **≤ 300 words** and leads with these four labelled beats, in order:

1. **What it is** — one sentence.
2. **When to pick this** — over the alternatives.
3. **When NOT to pick this.**
4. **Real venue** — a production user, or "no production user known".

Avoid jargon without an inline definition. A Hacker News reader with no finance background
should be able to follow it.

### Required section markers (enforced by lint)

Every catalog entry file MUST contain, literally:

- A level-1 title line beginning with `# `
- A ```` ```mermaid ```` fenced block
- `**What it is.**`
- `**When to pick this.**`
- `**When NOT to pick this.**`
- `**Real venue.**`
- `**Recommended crate.**` (name a crate from the inventory, or `none — std` / `n/a`)

## 3. Rust crates inventory

Master inventory of crates the generator may emit. All are permissively licensed
(MIT/Apache-2.0/BSD/Zlib) — **no GPL/LGPL** — and audited with `cargo audit`
(0 open RUSTSEC advisories as of 2026-05-29). Re-run `cargo audit` before each release.

| Crate | License | Health | Category it supports |
|-------|---------|--------|----------------------|
| `disruptor` | MIT/Apache-2.0 | active | LMAX/DMAX ring-buffer concurrency |
| `rtrb` | MIT/Apache-2.0 | active | SPSC ring buffer |
| `ringbuf` | MIT/Apache-2.0 | active | lock-free ring buffer |
| `crossbeam` | MIT/Apache-2.0 | active | channels, deque, skiplist, epoch GC |
| `dashmap` | MIT | active | concurrent hashmap |
| `flurry` | MIT/Apache-2.0 | maintained | concurrent hashmap (Java-CHM port) |
| `papaya` | MIT/Apache-2.0 | active | lock-free concurrent hashmap |
| `parking_lot` | MIT/Apache-2.0 | active | faster mutex/rwlock |
| `rust_decimal` | MIT | active | deterministic money arithmetic |
| `fixed` | MIT/Apache-2.0 | active | fixed-point arithmetic |
| `ahash` | MIT/Apache-2.0 | active | fast non-crypto hash |
| `fxhash` | MIT/Apache-2.0 | ⚠ unmaintained (RUSTSEC-2025-0057) | fast non-crypto hash — **do not emit**; prefer `ahash`/`rustc-hash`. Present only transitively via `inquire` (dev CLI). |
| `bumpalo` | MIT/Apache-2.0 | active | bump/arena allocator |
| `typed-arena` | MIT | stable | typed arena allocator |
| `slab` | MIT | active | slab allocator / id reuse |
| `tokio-uring` | MIT | maintained | io_uring async runtime |
| `glommio` | MIT/Apache-2.0 | maintained | thread-per-core io_uring runtime |
| `monoio` | MIT/Apache-2.0 | active | thread-per-core io_uring runtime |
| `quanta` | MIT/Apache-2.0 | active | high-precision timestamps |
| `minstant` | MIT/Apache-2.0 | active | low-overhead `Instant` |
| `tracing` | MIT | active | sampled observability off the hot path |
| `criterion` | MIT/Apache-2.0 | active | statistical benchmarking |
| `iai-callgrind` | MIT/Apache-2.0 | active | instruction-count benchmarking |
| `loom` | MIT | active | exhaustive concurrency model checking |
| `shuttle` | MIT/Apache-2.0 | active | randomized concurrency testing |
| `clap` | MIT/Apache-2.0 | active | CLI argument parsing |
| `inquire` | MIT | active | interactive wizard prompts |
| `tera` | MIT | active | codegen templates |
| `ratatui` | MIT | active | terminal UI (visualization v0.1) |
| `axum` | MIT | active | web server (visualization v0.2) |
| `serde` | MIT/Apache-2.0 | active | (de)serialization at boundaries |
| `thiserror` | MIT/Apache-2.0 | active | error types |

## 4. Worked examples per category

One reference entry per category demonstrates the standard in action:

- Matching — `docs/catalog/matching/price-time-fifo.md`
- Books — `docs/catalog/books/btreemap.md`
- Concurrency — `docs/catalog/concurrency/lmax-disruptor.md`
- Risk — `docs/catalog/risk/isolated-margin.md`
- Liquidation — `docs/catalog/liquidation/partial-liquidation.md`
- Perps — `docs/catalog/perps/funding-rate.md`
- Prediction MM — `docs/catalog/prediction/lmsr.md`
- Infra — `docs/catalog/infra/write-ahead-log.md`
