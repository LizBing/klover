# Klover top-level build orchestrator.
#   make / make all   — core + rust
#   make core         — libklover-core (CMake)
#   make rust         — klover crate (needs core)
#   make classes      — javac --release 8 → test_data/classes/
#   make test         — Java test classes + CTest + cargo test
#   make test-simple  — run the SimpleAddition end-to-end test
#   make clean

.DEFAULT_GOAL := all

.PHONY: all core rust classes verify-classes check test test-c test-rust \
	test-simple clean compile-commands help

BUILD_DIR      ?= build
BUILD_TYPE     ?= Debug
CORE_DIR       := $(BUILD_DIR)/core
CARGO_TARGET   ?= $(abspath $(BUILD_DIR)/cargo)
CMAKE          ?= cmake
CARGO          ?= cargo
JAVAC          ?= javac

# Keep the native and Rust sides on the same build profile.  Debug-like CMake
# configurations use Cargo's default dev/test profiles; release-like CMake
# configurations use Cargo's release profile.
ifneq ($(filter Release RelWithDebInfo MinSizeRel,$(BUILD_TYPE)),)
CARGO_PROFILE_FLAG := --release
endif

ROOT           := $(abspath .)
KLOVER_CORE_DIR := $(abspath $(CORE_DIR))

CMAKE_FLAGS    := -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) -DCMAKE_C_COMPILER=clang
CARGO_FLAGS    := --manifest-path rust/Cargo.toml $(CARGO_PROFILE_FLAG)
CARGO_ENV      := KLOVER_CORE_DIR=$(KLOVER_CORE_DIR) CARGO_TARGET_DIR=$(CARGO_TARGET)

NPROC          := $(shell sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 4)

TEST_JAVA_SRCS := $(wildcard test_data/classes/*.java)
OBJECT_JAVA    := java/java.base/java/lang/Object.java
CLASSES_OUT    := test_data/classes

help:
	@echo "Targets: all core rust classes verify-classes check test test-c test-rust test-simple clean compile-commands"
	@echo "Vars:    BUILD_DIR=$(BUILD_DIR) BUILD_TYPE=$(BUILD_TYPE)"

all: core rust

# --- C core (CMake) ----------------------------------------------------------

core:
	$(CMAKE) -S $(ROOT) -B $(CORE_DIR) $(CMAKE_FLAGS)
	$(CMAKE) --build $(CORE_DIR) -j$(NPROC)

# --- Rust --------------------------------------------------------------------

rust: core
	$(CARGO_ENV) $(CARGO) build $(CARGO_FLAGS)

check: core
	$(CARGO_ENV) $(CARGO) check $(CARGO_FLAGS) --all-targets

# --- Java 8 test classes (write back into test_data/classes/) ----------------

classes:
	@command -v $(JAVAC) >/dev/null || { echo "error: javac not found"; exit 1; }
	@mkdir -p $(CLASSES_OUT)
	$(JAVAC) --release 8 -d $(CLASSES_OUT) $(TEST_JAVA_SRCS) $(OBJECT_JAVA)
	@$(MAKE) verify-classes

verify-classes:
	@python3 scripts/verify-class-major.py 52 \
		$(CLASSES_OUT)/ArithmeticOps.class \
		$(CLASSES_OUT)/ConstantOps.class \
		$(CLASSES_OUT)/LdcWideOps.class \
		$(CLASSES_OUT)/ControlFlow.class \
		$(CLASSES_OUT)/ReferenceLoads.class \
		$(CLASSES_OUT)/StoreOps.class \
		$(CLASSES_OUT)/SimpleAddition.class \
		$(CLASSES_OUT)/java/lang/Object.class \
		$(CLASSES_OUT)/Arith.class

# --- Tests -------------------------------------------------------------------

test-c: core
	ctest --test-dir $(CORE_DIR) --output-on-failure

test-rust: core classes
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS)

test-simple: core classes
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS) \
		--test test_simple_addition -- --nocapture

test: classes test-c test-rust

# --- Misc --------------------------------------------------------------------

compile-commands: core
	@ln -sfn $(CORE_DIR)/compile_commands.json $(ROOT)/compile_commands.json
	@echo "linked compile_commands.json -> $(CORE_DIR)/compile_commands.json"

clean:
	rm -rf $(BUILD_DIR)
	rm -f $(ROOT)/compile_commands.json
	find $(CLASSES_OUT) -name '*.class' -type f -delete 2>/dev/null || true
	$(CARGO) clean $(CARGO_FLAGS) 2>/dev/null || true
