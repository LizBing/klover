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
make test-simple          # isolated SimpleAddition interpreter smoke test
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

The interpreter smoke test runs in a child process because it initializes global
VM state independently of the metaspace allocator unit tests.

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
cargo test --manifest-path rust/Cargo.toml --locked
```

Without `KLOVER_CORE_DIR`, the build script uses `build/debug/core` or
`build/release/core` according to Cargo's profile. Tests default to
`build/test-classes`; set `KLOVER_TEST_CLASSES` when running Cargo directly against
fixtures in a custom directory. Native runtime paths reference the build directory;
these are development artifacts, not a relocatable distribution. Cross-compilation
requires matching native toolchain/linker configuration and is not automated here.
