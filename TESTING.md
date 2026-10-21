# Testing Klover

Run `make test` from the repository root. Make builds C, compiles Java 8
fixtures, runs CTest and Cargo, and checks Rust ownership examples. A C11
compiler, CMake/CTest 3.20+, Rust 1.85+, GNU Make, and a JDK supporting
`javac --release 8` are required for the complete suite. No test framework
dependencies have been added.

## Commands

| Command | Scope |
| --- | --- |
| `make test` | All required C, Rust, Java execution, and ownership checks |
| `make test PROFILE=release` | The same suite with optimized C and Rust |
| `make test-core` | C tests; no Rust or Java compiler needed |
| `make test-rust` | C prerequisite, Java fixtures, and Rust runtime tests |
| `make test-compile` | One valid ownership example and four required compiler rejections |
| `make test-classes` | Compile the minimal Java library and Java test programs |
| `make test-sanitize` | ASan/UBSan on compatible C tests, then UBSan on all C tests |

`JOBS`, `CMAKE`, `CTEST`, `CARGO`, and `JAVAC` have the same meaning as in
the normal build. For example, `make test JOBS=2 JAVAC=/path/to/jdk/bin/javac`.
The normal `make` library build does not need a JDK.

For a focused Rust test without preparing Java fixtures:

```sh
make build-core
cargo test --manifest-path rust/Cargo.toml --locked --target-dir build/cargo \
  --lib engines::interpreter::tests
```

To run a single Java execution test:

```sh
make build-core test-classes
KLOVER_TEST_CLASSES="$PWD/build/test-classes" \
  cargo test --manifest-path rust/Cargo.toml --locked --target-dir build/cargo \
  --lib tests::execution::java_arithmetic_and_wide_arguments -- --exact
```

For C, use `ctest --test-dir build/debug/core -R '^virt_space$' --output-on-failure`.
For verbose output, add `-V`. The harness self-test deliberately creates one
failure inside its executable; a CMake wrapper verifies that it reports exactly
one failure and one success. CTest reports this self-test as successful only
when both accounting and output are correct.

## What belongs where

| Level | Location | Responsibility |
| --- | --- | --- |
| C components | `core/tests/` | Alignment, pointers and headers, virtual space, heap allocation |
| Rust components | Existing inline tests and `rust/src/tests/interpreter.rs` | Memory ownership, symbols, instruction semantics and frame state |
| Integration | `rust/src/tests/` | VM lifecycle, bootstrap loading, invocation and dispatcher behavior |
| Java programs | `tests/java/` | Source → class file → loader → metadata/lowering → dispatcher → return value |
| Compiler checks | `rust/compile/ownership.rs`, `tests/check-ownership.sh` | Lifetimes rejected by Rust's borrow checker |

The interpreter tests are included as a child of its private module. Larger
integration tests are crate-internal. Production modules remain private; no
public test API or runtime reset mechanism was introduced.

`support.rs` provides a small class-file builder, parsing and execution helpers,
temporary directories, and process isolation. Its class-file builder emits
only a single static method in a root class. Prefer explicit bytecode for exact
instruction forms and malformed inputs. Prefer Java source for complete
execution scenarios. Do not extend the helper into a general assembler unless
tests demonstrate a concrete need.

## Isolation and limits

Arguments, metaspace, bootstrap class state, and the native heap are process-wide.
Ordinary independent component tests run in parallel. Tests needing a fresh VM
re-execute one exact Cargo test in a child process. This covers initialization,
different class paths, and complete Java execution. C exhaustion and concurrent
allocation tests have separate executables for the same reason.

The Rust parent captures the child output, checks its exit status, rejects an
empty test selection, and kills/reaps a child after 20 seconds. CTest limits
each C test to 30 seconds. Java execution also has a finite polling limit, so
accidental loops fail with method and argument context. A test verifies that a
failing child propagates to its parent.

Budget slicing is tested with small slices and a large uninterrupted slice.
The current contract allows an instruction to start whenever any budget remains;
charging saturates at zero afterward. `iadd` currently costs two units. Tests
preserve that behavior rather than redefining scheduling policy.

Fixtures are regenerated under `build/test-classes/` on each Make test request,
including the project's minimal `java/lang/Object`. Deleted sources cannot leave
stale classes. Compilation uses `--release 8 -g:none`; no class binaries are
committed, and Cargo's build script never invokes Java. A missing JDK or missing
fixture fails clearly. Constructors in Java fixtures are parsed but not executed;
the current tests invoke supported static methods directly.

## Assertions and coverage

Test a behavior or invariant with an independently chosen expectation. Include
normal values, boundary values, and the documented error path. A table of
related numeric cases is preferable to a large generic testing abstraction.
Examples covered by the suite include:

- Integer overflow, operand order, signed division/remainder, and masked shifts.
- Floating arithmetic, NaN comparisons/conversions, infinities, and signed zero.
- All legal `dup` forms, invalid stack shapes, wide local overlap, typed returns.
- Taken/untaken branches, loops, dense/sparse switches, and budget resumption.
- Loader errors, superclass loading, cache identity, and recovery from a missing superclass.
- VM argument publication, rejection of repeated initialization, concurrent
  initialization, retry after failure, and reuse of independently initialized metaspace.
- Heap payload zeroing, exact capacity, OOM state preservation, and concurrent non-overlap.
- Virtual-space rounding/accounting, overflow rejection, and preserving occupied mappings.

Use the [JVMS 8 instruction specification](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html)
for expected JVM semantics. Match structured errors and values rather than log
wording. Java division by zero currently returns Klover's `DivisionByZero`
execution error; Java exception dispatch is not implemented.

The previous `compile_fail` doctests accepted private-module import failures.
They were replaced by an example target that includes the real production
modules in the same crate. The ownership script first requires a valid case to
compile. It then requires E0505 for dropping an allocator, owner, or registry
while borrowed, and E0502 for mutation during a shared borrow. Unexpected error
codes or unexpectedly successful compilation fail the target. Production type
definitions are not copied or made public.

See [README.md](README.md#current-capabilities) for the supported execution
subset and its limits. Passing tests do not establish complete JVM conformance.
Tracing/reclamation, object instructions, method-call bytecodes, class
initialization, verification, and Java exceptions need tests as they become
implemented. Reference identity checks currently cover null references; the
suite does not fabricate live Java objects. Reference-JVM comparisons, fuzzing,
performance gates, and coverage-percentage targets are future additions.

## Sanitizers and CI

`make test-sanitize` uses separate directories under `build/sanitize/`, so
instrumented archives cannot enter a normal Rust link. It runs:

1. AddressSanitizer plus UndefinedBehaviorSanitizer for tests labelled
   `address-compatible`: harness, alignment, object-model helpers, virtual space.
2. UndefinedBehaviorSanitizer for the complete C suite, including the heap.

AddressSanitizer reserves shadow memory that conflicts with Klover's fixed heap
address on the tested macOS host. Heap tests therefore remain required under
normal builds and UBSan; ASan's compatible selection is explicit and consistent
across hosts. Allocations inside a custom mapped heap also need logical boundary
assertions; ASan alone cannot establish those boundaries. Rust and mixed-language
sanitizer instrumentation are not enabled by this target.

GitHub Actions runs debug tests on Linux and macOS, release tests on Linux, and
the C sanitizer target on Linux. CI uses the same Make targets as local testing,
with job-level deadlines. The workflow takes effect when these changes are
pushed; local verification does not establish that hosted jobs have passed.

## VM initialization contract

`vm::try_init` publishes the VM arguments only after metaspace and the native
heap initialize successfully. Every returned error gives the caller ownership
of the submitted arguments. Metaspace may already be available from independent
allocator use or a previous attempt whose native heap initialization failed.
It is reused without discarding existing allocations.

Once VM initialization succeeds, another call returns `AlreadyInitialized`,
whether its arguments are identical or different. Rejected calls preserve the
published arguments. Concurrent VM initialization permits exactly one success.
This contract covers initialization; it does not provide VM shutdown, reset,
or multiple VM instances within a process.

The lifecycle regressions are required in both debug and release suites:

| Scenario | Required result |
| --- | --- |
| Successful initialization | Publish the successful arguments; reject identical and different subsequent requests |
| Failed native heap initialization | Return `GcInitFailed` and the submitted arguments without publishing a VM |
| Retry after failed native heap initialization | Reuse metaspace, publish the retry's arguments, then reject further initialization |
| Independently initialized metaspace | Start the VM while preserving existing allocations and permitting further allocation |
| Concurrent initialization | Exactly one caller succeeds; the others receive `AlreadyInitialized` |

Run the lifecycle tests without Java fixtures:

```sh
make build-core
cargo test --manifest-path rust/Cargo.toml --locked --target-dir build/cargo \
  --lib tests::vm_lifecycle
```

To select only the retry regression, replace the filter with
`tests::vm_lifecycle::retry_after_failed_gc_initialization` and append
`-- --exact`. No `--ignored` flag is needed. For release, build the C core with
`PROFILE=release` and add `--release` to the Cargo command.
