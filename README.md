# Klover

A small HotSpot-inspired JVM (C core + Rust). MVP target: **JVMS 8**
(classfile major 52). The current interpreter supports a limited JVM subset.

## Requirements

- C11 compiler, CMake 3.20+ (including CTest), and GNU Make
- Rust 1.85+ (edition 2024)
- JDK 9+ with `javac --release 8`, for the full test suite and Java compilation

The C core currently uses POSIX memory mapping and 64-bit addresses. Use a
64-bit macOS or Linux host; Windows and cross-compilation are not supported by
this build setup.

## Commands

Run from the repository root, or use `make -C /path/to/klover`.
If upgrading from the previous shared-library build, run `make clean` once to
remove old libraries before building.

```sh
make                           # Build the C core and Rust library in debug mode
make test                      # Run C, Rust, Java execution, and ownership tests
make classes                   # Compile java/java.base for Java 8
make PROFILE=release           # Build optimized C and Rust libraries
make test PROFILE=release      # Run tests with release libraries
make test-sanitize             # Run supported C sanitizer checks
make JOBS=8                    # Allow each build tool up to eight workers
make clean                     # Remove build/ for both profiles and Java
make help                      # List commands and settings
```

For work on one language, use `make build-core`, `make build-rust`,
`make test-core`, or `make test-rust`. Rust commands always build the C core
first. Full Rust testing also prepares Java test fixtures. The normal native
library build does not require Java. See [TESTING.md](TESTING.md) for test
organization, focused commands, isolation, and the documented runtime regression.

`PROFILE` accepts only `debug` and `release`. `JOBS` defaults to `4` and must be
a positive integer. These limits apply per tool; `make -j` can run the C and
Rust test groups concurrently. Executable paths can be overridden with
`CMAKE`, `CTEST`, `CARGO`, and `JAVAC`, for example:

```sh
make classes JAVAC=/path/to/jdk/bin/javac
```

There is currently no executable launcher; `make` produces libraries.

## How the build works

The root Makefile defines the order of operations. CMake owns the C source
list and C tests; Cargo owns Rust compilation and Rust tests. Both tools track
dependencies and reuse unchanged outputs. Cargo commands use `--locked`, so
building does not update `rust/Cargo.lock`.

CMake builds `libklover-core.a`. Make passes its directory to Cargo through
`KLOVER_CORE_DIR`; `rust/build.rs` links the archive and tells Cargo to relink
when it changes. Static linking avoids runtime shared-library path setup.
The build script does not compile C or search for another library if the
selected one is missing.

| Output | Location |
| --- | --- |
| C debug library and test executables | `build/debug/core/` |
| C release library and test executables | `build/release/core/` |
| Rust debug libraries and tests | `build/cargo/debug/` |
| Rust release libraries and tests | `build/cargo/release/` |
| Java classes, with package directories | `build/classes/` |
| Java test fixtures and bootstrap classes | `build/test-classes/` |
| C sanitizer builds | `build/sanitize/` |

Cargo calls its debug compilation profile `dev`; the Makefile maps
`PROFILE=debug` to that name. Both native profiles can coexist.

`make classes` recompiles the small Java library together and clears its old
output first, so removed sources cannot leave stale class files. Its compiler
argument file is saved as `build/classes/sources.list`.

## Direct tool use

The Makefile is the normal entry point. For a focused Rust command, build C
first and explicitly select its absolute directory:

```sh
make build-core test-classes
KLOVER_CORE_DIR="$PWD/build/debug/core" \
KLOVER_TEST_CLASSES="$PWD/build/test-classes" \
  cargo test --manifest-path rust/Cargo.toml --locked --target-dir build/cargo
```

For release, use `make build-core PROFILE=release`, set the directory to
`$PWD/build/release/core`, and add `--release` to Cargo. Direct Cargo commands
require this setting, including `cargo check`.

To select a different C compiler, configure a fresh CMake directory with `CC`:

```sh
make clean
CC=clang make
```

CMake remembers its compiler in each profile's build directory. Changing
`CC` after configuration requires cleaning or creating a fresh build directory.
`make clean` removes only `build/`; outputs from direct commands that used
other directories must be cleaned separately.
