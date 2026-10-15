# Contributing to Market Forge

Thanks for your interest! Market Forge is a Rust CLI + SDK that generates tailored
matching-engine venues from your design choices.

## Relationship to OrderBook-rs

Market Forge's design lineage traces to **OrderBook-rs** by Joaquín Béjar García
(<https://github.com/joaquinbejar/OrderBook-rs>), whose LMAX/DMAX Disruptor order book is the
inspiration for our default concurrency template. **Market Forge is a native
re-implementation, not a vendored fork** — all code here is original and dual-licensed
MIT/Apache-2.0. See `NOTICE.md` for attribution.

**Upstream-sync policy:** because we re-implement rather than vendor, there is no automatic
upstream merge. We review OrderBook-rs releases for good ideas on a best-effort basis (target:
a quarterly look for the first year) and credit any adopted design in `NOTICE.md`. If you port
a specific upstream improvement, note it in your PR description and the NOTICE file.

## Dev setup

```bash
rustup show                            # toolchain pinned by rust-toolchain.toml (1.94)
cargo build --workspace
cargo test --workspace
git config core.hooksPath .githooks    # enable the agent-doc drift pre-commit hook
```

The repo is **pure Rust** — there is no Node/Bun toolchain. Dev chores run through
`cargo xtask`:

- `cargo xtask sync-ai` — regenerate the per-tool agent docs (`AGENTS.md`, `CLAUDE.md`, …)
  from the hand-edited sources under `.ai/`.
- `cargo xtask check-sync` — regenerate, then fail on drift (what the pre-commit hook runs).
- `cargo xtask lint-catalog` — lint `docs/catalog/` against `docs/standards.md` §2.

## Definition of Done (every change)

- `cargo check --workspace --all-targets` clean
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- `cargo test --workspace` green; SDK/engine crates ≥ 80% covered
- `cargo audit` clean (no open RUSTSEC advisories)
- If you edited `.ai/` sources, regenerate agent docs (`cargo xtask check-sync` clean)
- If you touched the generator, regenerate a sample venue and prove it
  `cargo build` + `cargo test` + `cargo bench` green.
- Money/size use `rust_decimal::Decimal`, never `f64`.
- No `unsafe` without a `// SAFETY:` comment.

## Repo layout

- `crates/mf-core` — domain types + `OrderBook`/`MatchingEngine` traits
- `crates/mf-book` — book data structures (BTreeMap, bitmap, …)
- `crates/mf-matching` — matching algorithms (FIFO, pro-rata, …)
- `crates/mf-concurrency` — concurrency runners (single-thread, Disruptor)
- `crates/mf-codegen` — `VenueSpec`, compatibility matrix, Tera rendering
- `crates/market-forge` — the `market-forge` CLI + interactive wizard
- `templates/` — Tera templates stamped into generated venues
- `docs/` — `architecture.md`, `standards.md`, and the algorithm `catalog/`

## Commits

Conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, …). Keep them atomic. The
maintainer handles pushing and any GitHub-side operations.

## Contribution workflow and AI assistance

This policy applies to [ogarciarevett/market-forge](https://github.com/ogarciarevett/market-forge).

All contributors and agents must work on a topic branch and submit changes by pull request to `main`. Only the human repository owner, after confirming exact GitHub account control, may choose direct changes to `main` or the default branch; existing protections still apply. Agents must never directly update those branches, even when authenticated with the owner's account. GitHub account authentication does not establish that a human is operating it.

AI assistance is welcome. In the PR description, disclose each tool, the model identifier when exposed, and the scope of its help. If the model is not exposed, say `unknown (not exposed)`; do not guess. This scoped disclosure is required even if an inherited instruction otherwise discourages tool attribution.

Example disclosure (replace the example with accurate details):

```text
AI assistance: tool = coding assistant; model = unknown (not exposed).
Scope: helped draft the change and tests.
Human responsibility: I reviewed and understand every submitted change.
Validation: list actual checks and results, or explain checks not run.
```

A named human must review and understand the complete change, verify its tests and claims, check security and privacy, and confirm the right to contribute all code and dependencies under the applicable licenses. AI assistance does not waive existing review, CLA/DCO, signing, coverage, or CI requirements. Follow the existing commit conventions, including Conventional Commits where required. No AI `Co-authored-by` trailer or one-line commit attribution is required; any required human credit or DCO sign-off still applies.

Do not include secrets, personal data, private prompts, conversation transcripts, or hidden reasoning in the disclosure. None is required. Provide only the tool/model, scope, human accountability, and actual validation evidence.
