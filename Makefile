# Klover build entry point: Make coordinates CMake, Cargo, and Java fixtures.
.DEFAULT_GOAL := all

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
BUILD_DIR ?= $(ROOT)/build
BUILD_TYPE ?= Debug
ifeq ($(BUILD_TYPE),Debug)
PROFILE := debug
else ifeq ($(BUILD_TYPE),Release)
PROFILE := release
CARGO_PROFILE_FLAG := --release
else
$(error BUILD_TYPE must be Debug or Release)
endif

# Make's abspath treats spaces as separators; preserve the directory as one path.
BUILD_ROOT := $(if $(filter /%,$(BUILD_DIR)),$(BUILD_DIR),$(CURDIR)/$(BUILD_DIR))
CORE_DIR := $(BUILD_ROOT)/$(PROFILE)/core
CARGO_TARGET ?= $(BUILD_ROOT)/cargo
CLASSES_OUT := $(BUILD_ROOT)/test-classes
CMAKE ?= cmake
CTEST ?= ctest
CARGO ?= cargo
JAVAC ?= javac
PYTHON ?= python3
JOBS ?= $(shell sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 4)
CMAKE_ARGS ?=
CARGO_FLAGS := --manifest-path "$(ROOT)/rust/Cargo.toml" --locked $(CARGO_PROFILE_FLAG)
CARGO_ENV = KLOVER_CORE_DIR="$(CORE_DIR)" CARGO_TARGET_DIR="$(CARGO_TARGET)" KLOVER_TEST_CLASSES="$(CLASSES_OUT)"

.PHONY: all core rust check classes verify-classes test test-c test-rust test-unit test-integration test-simple test-build clean compile-commands help

all: rust

core:
	$(CMAKE) -S "$(ROOT)" -B "$(CORE_DIR)" -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) $(CMAKE_ARGS)
	$(CMAKE) --build "$(CORE_DIR)" --parallel $(JOBS)

rust: core
	$(CARGO_ENV) $(CARGO) build $(CARGO_FLAGS)

check: core
	$(CARGO_ENV) $(CARGO) check $(CARGO_FLAGS) --all-targets --features integration-tests

classes:
	$(PYTHON) "$(ROOT)/scripts/build-test-classes.py" --javac "$(JAVAC)" --output "$(CLASSES_OUT)"

verify-classes: classes
	$(PYTHON) "$(ROOT)/scripts/verify-class-major.py" 52 "$(CLASSES_OUT)"

test-c: core
	$(CTEST) --test-dir "$(CORE_DIR)" --output-on-failure

test-rust: core classes
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS) --features integration-tests

test-unit: core
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS) --lib

test-integration: core classes
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS) --features integration-tests --test jvm

test-simple: core classes
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS) --features integration-tests --test jvm simple_addition -- --nocapture

test-build:
	$(PYTHON) -B -m unittest discover -s "$(ROOT)/scripts/tests" -v

test: test-c test-rust test-build

compile-commands: core
	ln -sfn "$(CORE_DIR)/compile_commands.json" "$(ROOT)/compile_commands.json"

clean:
	$(PYTHON) "$(ROOT)/scripts/clean-build.py" "$(BUILD_DIR)" "$(CARGO_TARGET)"

help:
	@echo "Targets: all core rust check classes verify-classes test test-c test-rust test-unit test-integration test-simple test-build clean compile-commands"
	@echo "Variables: BUILD_TYPE=Debug|Release BUILD_DIR=... CARGO_TARGET=... JOBS=..."
	@echo "Tools: CMAKE CTEST CARGO JAVAC PYTHON; extra CMake options: CMAKE_ARGS"
