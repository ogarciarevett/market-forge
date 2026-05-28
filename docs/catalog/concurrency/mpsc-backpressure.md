# MPSC with Back-Pressure

```mermaid
flowchart LR
    producer_one[Producer One] -->|send| bounded_queue[Bounded Queue]
    producer_two[Producer Two] -->|send| bounded_queue
    producer_three[Producer Three] -->|send| bounded_queue
    bounded_queue -->|receive| consumer[Single Consumer]
    bounded_queue -->|full| block_sender[Block Sender]
```

**What it is.** A multi-producer single-consumer queue with a fixed capacity: when it fills up, senders are forced to wait ("back-pressure"), which stops fast producers from overwhelming a slower consumer.

**When to pick this.** Several threads submit work (orders, events) to one processor and you want memory bounded and the system to self-throttle under load instead of exploding.

**When NOT to pick this.** Producers must never block (a real-time audio or kernel path), or you genuinely need multiple consumers sharing the stream.

Memory is capped at `capacity` messages; a bounded queue trades some sender stalls for a hard upper bound, unlike an unbounded queue that can grow until it crashes.

**Real venue.** Used across trading and streaming backends; no production user known for this specific catalog entry.

**Recommended crate.** crossbeam
