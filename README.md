# Klover

A small HotSpot-inspired JVM (C core + Rust). MVP target: **JVMS 8** (classfile major 52).

## Dependencies

- clang, cmake, ctest
- Rust toolchain (see `rust/Cargo.toml` edition)
- JDK 8+ with `javac` (classes are always built with `--release 8`)

## Build

```bash
make              # Debug: core (CMake) + rust (Cargo)
make core         # libklover-core → build/core/
make rust         # klover crate (automatically builds core)
make classes      # javac --release 8 → test_data/classes/
make check        # cargo check --all-targets
make test         # build Java classes, then run CTest + all Rust tests
make test-c
make test-rust
make test-simple  # run the SimpleAddition end-to-end interpreter test
make clean
```

Useful variables:

```bash
make BUILD_TYPE=Release  # CMake Release + Cargo --release
make BUILD_DIR=build
```

The top-level Makefile is the canonical build entry point. It keeps the C and
Rust build profiles aligned, puts Cargo artifacts under `build/cargo`, and
prepares Java test classes before Rust integration tests.

Rust discovers the C library via `KLOVER_CORE_DIR` (set by the Makefile to `build/core`).
You can also build manually:

```bash
cmake -S . -B build/core -DCMAKE_C_COMPILER=clang
cmake --build build/core
export KLOVER_CORE_DIR=$PWD/build/core
cargo build --manifest-path rust/Cargo.toml
```

`.class` files are **not** committed; run `make classes` after clone.

## Docker

```bash
docker compose build
docker compose run --rm dev make all classes test-c
```
