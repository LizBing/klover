# Klover

A small HotSpot-inspired JVM (C core + Rust). MVP target: **JVMS 8**
(classfile major 52). The current interpreter supports a limited JVM subset.

## Requirements

- C11 compiler, CMake 3.20+, CTest, and Make
- Rust 1.85+ (edition 2024)
- JDK 9+ with `javac --release 8`; a HotSpot runtime for differential tests
