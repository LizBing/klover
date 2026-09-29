# Klover

A small HotSpot-inspired JVM (C core + Rust). MVP target: **JVMS 8**
(classfile major 52). The current interpreter supports a limited JVM subset.

## Requirements

- C11 compiler, CMake 3.20+, CTest, and Make
- Rust 1.85+ (edition 2024)
- JDK 9+ with `javac --release 8`; a HotSpot runtime for differential tests
- Python 3.9+, with no additional packages

## Build and test

Run from the repository root, or use `make -C /path/to/klover`.

| Component | Build only | Run tests |
|---|---|---|
| C core | `make build-core` | `make test-core` |
| Rust | `make build-rust` | `make test-rust` |
| Differential | `make build-differential` | `make test-differential` |
| Everything | `make build-all` (also plain `make`) | `make test-all` |

Each test target builds its prerequisites automatically and stops on build
failure. Build targets never execute tests. `build-core` includes the native test
executables; `build-rust` builds the Rust library; `build-differential` builds the
Rust invocation adapter and compiles the shared Java fixtures and HotSpot wrapper.
`test-rust` runs fixture-builder checks plus Rust unit and JVM integration tests.
`test-differential` runs harness checks followed by the HotSpot comparison.

```sh
make
make test-all
make test-rust BUILD_TYPE=Release
make test-all BUILD_TYPE=Release
make test-differential DIFF_ARGS='--filter arithmetic/*'
make clean
```

`BUILD_TYPE` accepts Debug (default) or Release. `JOBS` controls native build
parallelism (default 4). Tool overrides are `CMAKE`, `CTEST`, `CARGO`, `JAVAC`,
`JAVA`, and `PYTHON`. For example:

```sh
make test-differential JAVA=/path/to/hotspot/bin/java JAVAC=/path/to/jdk/bin/javac
```

All Make outputs use fixed repository-local paths:

```text
build/
  debug/core/                 C Debug library and test executables
  release/core/               C Release library and test executables
  cargo/debug/                Rust Debug artifacts
  cargo/release/              Rust Release artifacts
  test-classes/               shared Java fixtures and reference wrapper
  switch-integration-classes/ generated Rust integration fixtures
  differential/report.json   latest differential results
```

CMake and Cargo retain their normal incremental builds. Java fixtures are
recompiled once per Make invocation that needs them, including `make -j4 test-all`.
The small builder recreates `build/test-classes`, runs `javac --release 8`, and
removes partial output on failure. There is no fixture cache, fingerprint file,
or separate class-version verification step. Rust and differential tests consume
these same compiled fixtures. Old `.class` files in the source tree are unused.

`make clean` removes `build/` and the optional root `compile_commands.json`
symlink. The former custom build-directory variables and overlapping target names
have been removed. For editor integration, CMake writes its compilation database
to `build/debug/core/compile_commands.json` (or the Release equivalent).

Cargo uses the committed lockfile. Make selects the matching C library explicitly
through `KLOVER_CORE_DIR`. Direct Cargo commands still work after the prerequisites
have been built; for example, to select one Rust integration test:

```sh
make build-differential
CARGO_TARGET_DIR="$PWD/build/cargo" \
cargo test --manifest-path rust/Cargo.toml --locked \
  --features integration-tests --test jvm simple_addition
```

Without an explicit `KLOVER_CORE_DIR`, Cargo's build script uses the corresponding
`build/debug/core` or `build/release/core` directory. Native runtime paths refer to
that build directory; these are development artifacts, not a relocatable package.

## Differential tests

Edit `tests/differential/cases.py` to add a case or review feature support. All Java
case sources live in `test_data/classes/`. For example, a row can invoke:

```python
case("arithmetic/add_overflow", "ArithmeticOps", "iadd", "(II)I",
     (2147483647, 1), "locals", "int_arithmetic")
```

The `case` helper adds the `primitive_entries` requirement and converts argument
values to strings. It does not compute an expected result: HotSpot is the oracle.
Keep methods deterministic and use parameters or locals where needed to avoid
javac folding away the operation being tested. Future categories can use new
case-name prefixes and Java fixtures.

The runner starts a fresh process on each side for each enabled case:

- `HotSpotRunner.java` reflects the declared public static method on HotSpot.
- `rust/examples/differential_invoke.rs` uses Klover's existing bootstrap loader,
  `Invocation`, and `ExecDispatcher`. It is gated by `integration-tests`.

The Java wrapper is compiled separately against the JDK APIs. Only HotSpot
executes it and uses its own bootstrap Object. Klover loads the repository's
minimal Object fixture. Entries must extend Object directly, have no interfaces
and no class initializer; the adapter checks this restriction. Klover does not
need Java main, reflection, boxing, or printing support.

Current coverage includes constants, primitive arithmetic/overflow, locals,
conversions, branches, loops, switches, and primitive arguments/returns. The
73 cases currently yield **68 PASS / 0 FAIL / 5 SKIP**. Skips cover bytecode static
calls, constant-pool loads, and Java exceptions. Objects, arrays, virtual calls,
class initialization, and GC behavior remain outside this suite's supported scope.

False feature requirements produce SKIP before launching either VM. Unknown
requirements and empty selections are errors. Unexpected unsupported instructions,
crashes, timeouts, or invalid output produce FAIL, even if both sides fail.
Do not disable supported features to conceal mismatches. Reference and exception
results require a protocol extension before those categories can be enabled.

Primitive descriptors use `Z/B/C/S/I/J/F/D`, with `V` allowed only for returns.
Integer arguments use decimal values; booleans use 0 or 1 and chars use unsigned
UTF-16 code units. Float/double arguments use strings containing exactly 8/16
lowercase hexadecimal IEEE bits, without `0x` (`"80000000"` is float negative zero).
Successful output is one tab-separated line:

```text
KLOVER_DIFF_V1<TAB>I<TAB>-2147483648
```

The fields are protocol version, return type, and value. Values use decimal
integers, IEEE hex bits, `nan`, or `void`. Floating comparisons preserve signed
zero and compare finite values exactly. NaN payloads/signs normalize to `nan`.
Extra stdout or an incorrect type/range is a failure.

Failures show both results, exit statuses, stderr, arguments, descriptors, and
shell-quoted reproduction commands. The simple JSON report keeps these details
and skip reasons; every run overwrites it. Exit codes are 0 for no failures,
1 for differential failures, and 2 for infrastructure/configuration errors.
A build failure stops before the runner and leaves any previous report unchanged.
Save a failing report and the corresponding source revision before further edits.
An optional `seed` field on a case is retained and printed for future generators.

After building, the runner can be used directly:

```sh
python3 -B tests/differential/run.py --filter 'loops/*' --quantum 1
python3 -B tests/differential/run.py --profile release
```

The runner accepts `--java`, `--profile`, `--filter`, `--timeout` (seconds per
process, default 10), `--fuel` (default 1,000,000), and `--quantum` (budget per poll,
default 1,000). Fuel follows the dispatcher's existing saturating budget charges;
it is an execution bound rather than an instruction count. Reaching either
execution limit fails the case.

## Rust test organization

Unit tests live beside their Rust modules. JVM integration tests are collected in
`rust/tests/jvm.rs`, with execution, algorithm, and switch coverage under
`rust/tests/jvm/`. They load real fixtures and exercise both one-unit and large
execution budgets. Handcrafted class files cover malformed and boundary switch
cases that javac cannot conveniently produce.

The integration tests share a VM within their test binary and serialize fixture
loading because bootstrap class definition is not atomic yet. Unit tests do not
load Java fixtures. VM initialization remains centralized in `vm::try_init`, and
metaspace initialization is shared through `ms_api::ensure_initialized`.
Differential cases have fresh process state instead.

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
