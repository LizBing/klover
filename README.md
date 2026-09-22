# Klover

A small HotSpot-inspired JVM (C core + Rust). MVP target: **JVMS 8** (classfile major 52).

## Dependencies

- C11 compiler (Apple Clang, Clang, or GCC), CMake 3.20+, CTest, Make
- Rust toolchain supporting edition 2024 (Rust 1.85+)
- JDK 9+ whose `javac` supports `--release 8` (JDK 17 or 21 works)
- Python 3.9+

## Build

The top-level Makefile coordinates CMake, Cargo, and Java test fixtures.

```bash
make                      # Debug C core + Rust library
make BUILD_TYPE=Release    # Release C core + Cargo --release
make core                 # C library and C tests
make rust                 # Rust library, after building the C core
make check                # cargo check --all-targets
make classes              # incrementally compile and verify Java 8 fixtures
make verify-classes       # explicitly recheck all generated class versions
make test                 # CTest + all Rust tests + build-tool regression tests
make test-c
make test-rust
make test-unit            # Rust unit tests only
make test-integration     # JVM class-file integration tests only
make test-simple          # SimpleAddition interpreter and dispatcher smoke tests
make test-build           # fixture lifecycle, version checks, and safe cleanup
make compile-commands     # point compile_commands.json at the selected C build
make clean                # remove configured build roots
```

Only `BUILD_TYPE=Debug` and `BUILD_TYPE=Release` are supported. Their C libraries
are isolated, so changing profile does not overwrite the other profile's library:

```text
build/
  debug/core/
  release/core/
  cargo/debug/
  cargo/release/
  test-classes/
```

Java sources remain in `test_data/classes/`; generated classes are never written
there. The fixture builder tracks source contents (including additions/deletions),
javac identity/version, compiler flags, and generated class contents. A successful
rebuild replaces the output directory, removing stale classes. Failed compilation
preserves the last successful output but fails the build. Every generated class,
including nested classes, must have major version 52.

Rust unit tests run in one process. JVM integration tests use one dedicated
Cargo test binary, so their VM state is shared within that binary. Metaspace initialization is shared through
`runtime::ms_api::ensure_initialized()`. VM tests use
`runtime::test_support::init_vm()` with one fixed configuration, without spawning
child processes. Unit tests do not load Java fixtures. JVM integration tests
initialize their own VM and serialize fixture class loading because bootstrap
class definition is not atomic yet.

`runtime::vm::try_init(args)` serializes VM initialization and publishes arguments
only after metaspace and GC initialization succeed. A second successful-VM init
attempt is rejected, even with identical arguments. Errors return the original
arguments and distinguish repeated initialization, metaspace failure (including
unknown native status codes), and GC failure. Metaspace errors are cached; GC
allocation failures leave the heap uninitialized and permit a subsequent attempt.
The native GC rejects heap sizes that are not word-aligned, exceed 32 GiB, or
cannot accommodate the null sentinel plus at least one word. Tests share VM state;
they do not receive fresh class/static-field state between test cases.

## Interpreter regression tests

`make test` runs the C tests, Rust tests, and build-tool tests. Run
`make test BUILD_TYPE=Release` to exercise the optimized C and Rust builds as well.
`make verify-classes` explicitly checks that all compiled fixtures target Java 8.

Interpreter tests load real class files through the bootstrap loader and execute
static entry methods through `Invocation` and `ExecDispatcher`. Value-returning
fixtures run with both one-unit and large execution budgets. Coverage includes
the supported methods in `Arith`, `ArithmeticOps`, `ControlFlow`, `ConstantOps`,
`StoreOps`, `ReferenceLoads`, `Wide`, `InstructionOps`, and `StaticCallee`, plus
the `SimpleAddition` interpreter and dispatcher smoke tests. `AlgorithmOps` adds iterative Fibonacci, GCD,
primality, bit counting, nested loops, polynomial evaluation, and an infinite loop
that must yield when its execution budget expires.

Tests also exercise overflow, signed zero, NaN, numeric conversions, division by
zero, and zero-budget polls interleaved with execution. Stack-instruction tests
cover category-1/category-2 forms directly. Reference fixtures currently use null
references. Unsupported constant-pool loads are tested as explicit execution
errors; method invocation, object allocation, arrays, and Java exception handling
are not covered as successful execution paths yet.

Switch regressions live in `rust/tests/jvm/switches.rs` and run real class files through
the bootstrap loader and `ExecDispatcher`. `SwitchOps.java` covers both switch
opcodes at all four alignments, cases/defaults, fall-through, and loops. Minimal
class-file fixtures cover direct backward switch targets, single-entry tables,
operand-stack preservation, and malformed operands. These generated fixtures live
beside the compiled fixture directory in `switch-integration-classes/<pid>/`.
The metaspace ownership checks live beside the allocator unit tests in
`rust/src/runtime/ms_api.rs`.

The `integration-tests` feature exposes only the subsystem paths needed by the
single JVM integration target. `make test`, `make test-rust`, and `make check`
enable it automatically. Use the separate targets when iterating:

```bash
make test-unit
make test-integration
make test-simple # Run both SimpleAddition smoke tests in the JVM integration target.
cargo test --manifest-path rust/Cargo.toml --locked --features integration-tests --test jvm
```

## Configuration

```bash
make test BUILD_DIR=/tmp/klover-build JOBS=8
make core CMAKE_ARGS='-DCMAKE_C_COMPILER=clang'
make core CMAKE_ARGS='-DBUILD_TESTING=OFF'  # library only
make test-c CMAKE_ARGS='-DBUILD_TESTING=ON'
```

`BUILD_DIR` defaults to this repository's `build/`. `CARGO_TARGET` can override the
Cargo output directory. Tool overrides: `CMAKE`, `CTEST`, `CARGO`, `JAVAC`, `PYTHON`.
Use a fresh build directory when changing the C compiler. CMake options persist
in that directory's cache. `make clean` removes both configured build roots,
including an overridden `CARGO_TARGET`, and removes the compile database symlink
only when it points into those roots. It refuses source directories. Old `.class`
files left in the source tree by earlier builds are unused.

Cargo commands use the committed lockfile (`--locked`). Make passes an explicit
`KLOVER_CORE_DIR`; an invalid explicit path fails instead of finding an older
library elsewhere. Manual builds are also supported:

```bash
cmake -S . -B build/debug/core -DCMAKE_BUILD_TYPE=Debug
cmake --build build/debug/core
make classes
KLOVER_CORE_DIR="$PWD/build/debug/core" \
KLOVER_TEST_CLASSES="$PWD/build/test-classes" \
CARGO_TARGET_DIR="$PWD/build/cargo" \
cargo test --manifest-path rust/Cargo.toml --locked --features integration-tests
```

Without `KLOVER_CORE_DIR`, the build script uses `build/debug/core` or
`build/release/core` according to Cargo's profile. Tests default to
`build/test-classes`; set `KLOVER_TEST_CLASSES` when running Cargo directly against
fixtures in a custom directory. Native runtime paths reference the build directory;
these are development artifacts, not a relocatable distribution. Cross-compilation
requires matching native toolchain/linker configuration and is not automated here.

## Module visibility

Rust subsystem boundaries are controlled by `mod` / `pub mod` in `lib.rs` and
`mod.rs`. Top-level modules are private to this crate in normal builds; the
`integration-tests` feature exposes the paths needed by external tests. Callers use concrete module
paths; there is no module-level re-export facade. Cross-module types and methods
use `pub`; `pub(super)` is reserved for members needed only by the parent subsystem.

Error/result records such as `VmInitError`, `LoadError`, `ExecError`, and
`StepOutcome` have a private `_private: ()` field. Their defining modules control
construction; exposed fields remain readable and movable (and mutable when owned).
Objects with related fields, such as `Invocation`, `Method`, descriptors, and
`Code`, keep their fields private and expose only needed read accessors. Types
that already have private fields do not need an additional construction marker.
Input values such as `Arguments` remain directly constructible. FFI layouts are
kept unchanged by this visibility convention.
