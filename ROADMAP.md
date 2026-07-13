# Roadmap

This file is the canonical project roadmap. Detailed notes belong under `docs/`; runnable evidence belongs under `labs/`.

## Phase 0 — Repository foundation

- [ ] Initialize the root Git repository.
- [x] Add the virtual Cargo workspace and pinned root toolchain.
- [x] Add repository guidance and the source-reading method.
- [ ] Add upstream source metadata and checkout automation.
- [ ] Add the research inbox, catalog, and digest templates.
- [ ] Add the first standard-library async lab package.
- [ ] Establish formatting, lint, and test commands against a real package.

## Phase 1 — Rust async contracts and compiler model

- [ ] Map the relevant `core`, `alloc`, and `std` modules from the pinned `rust-src` component.
- [ ] Study and implement small examples for `Future`, `Poll`, `Context`, `Waker`, `RawWaker`, `Wake`, `Pin`, and `Unpin`.
- [ ] Separate the language-level approximate desugaring from the compiler-generated future state machine.
- [ ] Inspect HIR/THIR/MIR with a separately pinned nightly toolchain.
- [ ] Study cancellation through dropping futures and resources.
- [ ] Study `Send`/`Sync` consequences of values held across `.await` points.
- [ ] Complete a second-pass code-quality review of the studied standard-library source.

## Phase 2 — Tokio and Mio

- [ ] Pin a Tokio release and its resolved Mio baseline.
- [ ] Build black-box labs for tasks, spawning, scheduling, time, I/O, synchronization, cancellation, and shutdown.
- [ ] Map Tokio public APIs to internal runtime modules and important symbols.
- [ ] Study Mio's polling model, platform backends, registration, event delivery, and poller wakeup.
- [ ] Trace readiness from the operating system through Mio and Tokio to a task `Waker`.
- [ ] Complete a second-pass code-quality review of each studied Tokio/Mio subsystem.

## Phase 3 — tiny-mio

- [ ] Decide and document the initial platform scope; default proposal is Linux-first.
- [ ] Implement non-blocking sockets and a minimal epoll wrapper.
- [ ] Implement events, tokens, interests, registration, and poller wakeup.
- [ ] Define and test safety and lifecycle invariants.
- [ ] Produce frozen milestone labs and an evolving `tiny-mio` crate.

## Phase 4 — tiny-runtime

- [ ] Implement `block_on`, task representation, ready queue, and custom waking.
- [ ] Implement spawning and join handles.
- [ ] Implement timers and runtime time driving.
- [ ] Connect `tiny-mio` readiness to task wakeups through a reactor/driver.
- [ ] Implement minimal async TCP.
- [ ] Implement cancellation, resource cleanup, and graceful shutdown.
- [ ] Add multithreaded scheduling only after the single-threaded invariants are understood and tested.

## Phase 5 — Companion crates

- [ ] Reassess whether `tiny-macros`, `tiny-util`, `tiny-stream`, and `tiny-test` have earned independent crate boundaries.
- [ ] Create only the companion crates justified by concrete use cases.

## Phase 6 — Paired async practices

- [ ] Build a capability matrix before selecting scenarios.
- [ ] Implement each scenario once with idiomatic Tokio and once with idiomatic `tiny-runtime`.
- [ ] Keep the observable contract and black-box tests aligned without hiding runtime differences behind a generic abstraction.
- [ ] Validate relevant historical research against the pinned runtime version and current source.
- [ ] Document cancellation points, ownership, backpressure, blocking boundaries, shutdown, testing, and observability.

## Cross-cutting code-quality track

For every substantial standard-library, Tokio, Mio, or project subsystem:

- [ ] First-pass note explains correctness, invariants, state, and failure behavior.
- [ ] Second-pass note explains API and module design choices.
- [ ] At least one design pattern is tested in a focused lab or consciously rejected with a reason.
- [ ] Our implementation records where it follows or deliberately differs from the upstream design.
- [ ] Performance or elegance claims are supported by evidence rather than reputation.

