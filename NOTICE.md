# NOTICE

Market Forge
Copyright (c) 2026 Omar Garcia

This product is licensed under the terms of either the MIT license or the
Apache License, Version 2.0, at your option. See `LICENSE-MIT` and
`LICENSE-APACHE`.

## Attribution — design lineage

Market Forge's default concurrency template (the **LMAX/DMAX Disruptor** pattern) and its
initial order-book design draw on the design and prior art of:

- **OrderBook-rs** by **Joaquín Béjar García** — <https://github.com/joaquinbejar/OrderBook-rs>

Market Forge is a **native re-implementation** inspired by that work, not a verbatim copy or
vendored fork. The matching, book, concurrency, codegen, wizard, and algorithm-catalog code in
this repository is original to Market Forge. Where a pattern is directly modelled on
OrderBook-rs (notably the Disruptor-style sequencer), it is credited in-line and here.

> Human-only follow-ups (the agent contract forbids remote/network actions): create the public
> GitHub fork/mirror if desired, confirm the upstream license, and give Joaquín a courtesy
> heads-up. See `CONTRIBUTING.md` → "Relationship to OrderBook-rs".

## Third-party crates

Generated venues and the forge itself depend on third-party crates inventoried in
`docs/standards.md`. Every inventoried crate is permissively licensed (MIT/Apache-2.0/BSD/Zlib)
and audited with `cargo audit`. No GPL/LGPL dependency is permitted, because that would
constrain downstream commercial use of generated code.
