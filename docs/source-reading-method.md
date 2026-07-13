# Source reading method

This project studies both how async systems work and how mature Rust code is engineered. Those goals run in parallel, but they should not compete for attention during the same reading pass.

## Why use two passes

Elegant code is usually the visible result of constraints that are easy to miss: public compatibility, pinning, cancellation, platform differences, scheduler contention, unsafe invariants, compile time, or feature combinations. Judging style before recovering those constraints encourages copying shapes without understanding their purpose.

The project therefore uses one immediate mechanism pass and one milestone-level engineering pass.

## Pass 1 — Semantics and correctness

Answer these questions first:

1. What contract does the API or internal component promise?
2. What state is stored, and what are the allowed state transitions?
3. Which values may move, be shared, be dropped, or outlive a poll call?
4. Who owns progress, and what causes the next poll or event?
5. What happens on `Pending`, cancellation, shutdown, panic, and partial initialization?
6. Which invariants justify every unsafe operation?
7. Which behavior is a correctness requirement and which is a performance policy?
8. Which target, feature flags, and upstream commit does the observation apply to?

The output of this pass is a mechanism note, a small diagram when useful, and a focused executable lab.

## Immediate style capture

During pass 1, record promising choices without concluding that they are universally good. Examples include a type that makes an invalid state unrepresentable, a narrow unsafe boundary, an unusually clear name, or a test helper that exposes a hidden invariant.

Each capture should include:

- source repository, commit, path, and symbol;
- the constraint it appears to address;
- a question to verify during pass 2.

## Pass 2 — Engineering and code quality

After the mechanism unit is understood, revisit it and ask:

1. Why is this a module, type, trait, method, or free function?
2. Which invariants are represented by types and which remain comments or runtime checks?
3. How do names reveal ownership, lifecycle, readiness, and direction of data flow?
4. Where are public API, internal policy, platform code, and unsafe code separated?
5. How are errors classified, enriched, converted, or intentionally hidden?
6. How do feature flags and conditional compilation avoid contaminating the main path?
7. What do tests protect: examples, contracts, state transitions, races, or regressions?
8. What complexity exists because of compatibility or scale and should not be copied into a tiny implementation?

The output of this pass is a short design review, not a catalogue of clever syntax.

## Pattern record

Reusable observations should use this shape:

```text
Pattern:
Source:
Context and constraint:
Mechanism:
Why it is effective:
Trade-offs:
When not to use it:
Experiment or evidence:
Decision for tiny-async-lab:
```

## Applying a pattern

Before carrying an upstream pattern into project code:

1. Reproduce the underlying problem in a small lab.
2. Implement the simplest correct local solution.
3. Compare it with the upstream pattern under the same constraints.
4. Adopt, simplify, or reject the pattern explicitly.
5. Add a test for the invariant that motivated the decision.

This avoids line-by-line imitation while still learning from high-quality standard-library, Tokio, and Mio code.

## Timing

Code-quality learning is therefore synchronous at a small scale and deferred at a deep scale:

- During first-pass reading, capture notable decisions immediately.
- After each mechanism unit, spend a short second pass on engineering choices.
- After each subsystem milestone, perform a deeper review across module boundaries.
- Before finalizing the corresponding project subsystem, compare the upstream design with our current needs and documented constraints.

