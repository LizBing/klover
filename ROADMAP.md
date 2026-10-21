# Klover: development roadmap

Prepared 2026-10-20; Milestone 0 verified against commit `0bd9e23` plus the
changes described below. Milestone 0 was explicitly approved and is complete.
Later milestones remain proposals. Architectural implementation requires explicit
approval under `AGENTS.md`; approval of one milestone does not approve later ones.

## Direction

The next substantial goal should be **Java methods calling other Java methods**.
This makes the existing interpreter useful for larger programs and establishes
the execution machinery needed for constructors and class initialization.

Assumption: the immediate priority remains an interpreter for a documented
Java 8 subset. A JIT, broad JDK compatibility, and performance optimization are
later goals. Revisit the sequence if that priority changes.

## Current position

- The build and test infrastructure already exists. Extend it for each feature.
- Java fixtures exercise arithmetic, conversions, branches, loops, switches,
  typed returns, and execution that pauses when its budget is exhausted.
- `JavaThread` owns a frame stack, and frames have `accept_return`, but the
  dispatcher currently treats every method return as top-level completion.
- Call instructions are unsupported. Method lookup currently finds declarations
  in one class; it is not yet a complete JVM method-resolution mechanism.
- Field metadata is a stub. Class initialization, executable object operations,
  Java exception dispatch, and heap reclamation remain future work.
- The native heap allocates objects but does not collect them. Rust object
  pointer encode/decode methods are unfinished.
- The initialization-retry regression is mandatory. VM lifecycle coverage also
  checks independently initialized metaspace and rejection of identical and
  different arguments after successful initialization.

The supported execution subset and its limits are recorded in
[README.md](README.md#current-capabilities). The initialization contract and
focused commands are recorded in [TESTING.md](TESTING.md#vm-initialization-contract).

## Milestone 0: establish an accurate baseline — complete

Reviewed the existing fix: VM initialization serializes callers, accepts an
already initialized metaspace, and publishes arguments only after native heap
initialization succeeds. The native heap rejects invalid sizes before reserving
memory and releases its reservation if initial expansion fails. No production
runtime changes were needed for this milestone.

Delivered:

- Enabled the retry regression in the normal suite; checked publication of the
  retry's arguments and rejection of another initialization after success.
- Checked that identical and different rejected requests return their submitted
  arguments and preserve the published VM configuration.
- Added an isolated test for independently initialized metaspace, including
  preservation of an existing allocation across startup and further allocation.
- Retained failed-initialization and concurrent-initialization coverage.
- Replaced stale regression documentation and documented current capabilities.

Local validation completed on 2026-10-21:

| Check | Result |
| --- | --- |
| Focused `tests::vm_lifecycle` Cargo test filter | 7 passed; 0 ignored |
| `make test` | 7 C tests and 86 Rust tests passed; 0 Rust tests ignored; ownership compiler checks passed |
| `make test PROFILE=release` | 7 C tests and 86 Rust tests passed; 0 Rust tests ignored; ownership compiler checks passed |
| `rustfmt --edition 2024 --check rust/src/tests/vm_lifecycle.rs` | Passed |
| `git diff --check` | Passed |

Sanitizers were not rerun because this milestone changes only tests and
documentation. Local results do not establish that hosted CI has passed.

## Milestone 1: complete dispatcher and frame lifecycle

Before implementation, agree on the invocation/return protocol, ownership of
arguments, thread completion, and behavior after an execution error.

- Let a frame request a method invocation through the execution protocol.
- Have the dispatcher push the callee, pop completed frames, deliver return
  values to callers, and resume the caller at its next instruction.
- Complete and terminate the thread only when its entry frame returns.
- Share the execution budget across calls and returns; retain current charging
  semantics. Define a finite call-depth limit with a structured error.
- Preserve metadata lifetimes and useful caller/callee error context.

Completion: focused dispatcher tests demonstrate nested calls, void and typed
returns, argument order including long/double slots, return delivery exactly
once, budget exhaustion at call boundaries, depth exhaustion, and polling after
completion. Synthetic frames can establish this contract before call bytecodes.

## Milestone 2: execute a deliberately limited invokestatic subset

Retain symbolic method references during bytecode lowering: owner class, name,
and descriptor. Resolve and validate targets before execution, then connect
`invokestatic` to the dispatcher protocol.

Initially support ordinary static methods declared by the referenced class,
using bootstrap loading. Explicitly reject unsupported resolution cases,
native/synchronized methods, and classes requiring unsupported initialization.
Do not silently skip `<clinit>` in a class or its superclass chain. These limits
also apply to direct entry-method invocation.

Completion: Java fixtures execute same-class calls, calls across two classes,
and bounded recursion. Results agree for tiny and large execution budgets.
Tests cover wrong invocation kind, missing targets, narrow primitive parameters
and returns, wide values, and explicit rejection of unsupported cases.

Trade-off: this delivers useful calls with a small scope, while remaining a
documented subset. Full resolution and initialization add substantial work;
expand them explicitly rather than claiming complete `invokestatic` support.

## Milestone 3: static fields and class initialization

Agree on field storage and a per-class initialization state model before coding.
Then add primitive static fields, default values and supported constant values,
`getstatic`/`putstatic`, and execution of `<clinit>` before supported active uses.
Cover superclass ordering, recursive initialization by the same Java thread,
initialization that pauses for budget exhaustion, and persistent failure state.

Completion: a Java fixture initializes a static counter once, calls helpers from
its initializer, and returns consistent results over repeated calls. Unsupported
constant kinds and initialization scenarios fail explicitly. Until Java
exceptions exist, initialization failures remain documented VM execution errors.
Multiple-thread initialization requires a separate approved concurrency design.

## Milestone 4: objects and instance execution

First agree on the C/Rust object-layout boundary: class identity, inherited field
offsets, alignment, pointer encoding, allocation failure, and reference ownership.
Implement checked pointer conversions and allocation bindings, instance fields,
`new`, constructor calls through `invokespecial`, and then a defined subset of
`invokevirtual` with superclass lookup and overriding.

Completion: Java fixtures create objects, run constructors, read/write inherited
fields, and demonstrate overriding. Test null receivers, zeroed fields, reference
identity, and heap exhaustion. Allocation remains bounded until collection exists.

## Later milestones and dependencies

1. Primitive arrays, then reference arrays and type checks. Validate lengths,
   bounds, defaults, and allocation overflow.
2. Java exceptions: retain exception tables and bytecode locations, implement
   handler lookup and frame unwinding, then map runtime failures to Java objects.
   This depends on objects and class relationships. Preserve the distinction
   between Java exceptions and internal VM errors.
3. GC roots and collection: enumerate live references in frames, static fields,
   and loader storage. Approve safepoints (places execution can pause safely for
   collection), root updates, and a collector before adding reclamation. Move
   this earlier if memory use becomes the limiting requirement.
4. Add a small command-line runner when useful. A static-method runner can follow
   milestone 2; standard `main(String[])` needs arrays and String support.
5. Broaden verification, method/interface resolution, loader concurrency, Java
   threading, native integration, and the Java library as separately scoped work.

## Delivery discipline

Split milestones into reviewable changes. Each feature needs component tests
for its contract and Java fixtures for complete execution, including execution
across budget pauses. Run `make test` and release tests for runtime changes;
run the sanitizer target when changing C memory or object-layout code.
Add reference-JVM comparisons for supported numeric results as calls grow.

The next approval decision is milestone 1's dispatcher contract. Milestones 2
and later are sequencing proposals, not preapproved architecture.
