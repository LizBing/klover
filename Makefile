# Run from the repository root (or use make -C /path/to/klover).
.DEFAULT_GOAL := build-all
override ROOT := $(CURDIR)

BUILD_TYPE ?= Debug
ifeq ($(BUILD_TYPE),Debug)
PROFILE := debug
else ifeq ($(BUILD_TYPE),Release)
PROFILE := release
CARGO_PROFILE_FLAG := --release
else
$(error BUILD_TYPE must be Debug or Release)
endif

CMAKE ?= cmake
CTEST ?= ctest
CARGO ?= cargo
JAVAC ?= javac
JAVA ?= java
PYTHON ?= python3
JOBS ?= 4
DIFF_ARGS ?=

CORE_DIR := $(ROOT)/build/$(PROFILE)/core
CARGO_FLAGS := --manifest-path "$(ROOT)/rust/Cargo.toml" --locked $(CARGO_PROFILE_FLAG)
CARGO_ENV = KLOVER_CORE_DIR="$(CORE_DIR)" CARGO_TARGET_DIR="$(ROOT)/build/cargo" KLOVER_TEST_CLASSES="$(ROOT)/build/test-classes"

.PHONY: build-core build-rust build-differential build-all test-core test-rust test-differential test-all _classes clean

build-core:
	$(CMAKE) -S "$(ROOT)" -B "$(CORE_DIR)" -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) -DBUILD_TESTING=ON
	$(CMAKE) --build "$(CORE_DIR)" --parallel $(JOBS)

build-rust: build-core
	$(CARGO_ENV) $(CARGO) build $(CARGO_FLAGS)

# Shared by both test groups: Make runs this once per invocation, including -j.
_classes:
	$(PYTHON) -B "$(ROOT)/scripts/build-test-classes.py" --javac "$(JAVAC)"

build-differential: build-core _classes
	$(CARGO_ENV) $(CARGO) build $(CARGO_FLAGS) --features integration-tests --example differential_invoke

build-all: build-rust build-differential

test-core: build-core
	$(CTEST) --test-dir "$(CORE_DIR)" --output-on-failure

test-rust: build-core _classes
	$(PYTHON) -B -m unittest discover -s "$(ROOT)/scripts/tests" -v
	$(CARGO_ENV) $(CARGO) test $(CARGO_FLAGS) --features integration-tests

test-differential: build-differential
	$(PYTHON) -B -m unittest discover -s "$(ROOT)/tests/differential" -p 'test_*.py' -v
	$(PYTHON) -B "$(ROOT)/tests/differential/run.py" --java "$(JAVA)" --profile $(PROFILE) $(DIFF_ARGS)

test-all: test-core test-rust test-differential

clean:
	rm -rf -- "$(ROOT)/build"
	@if [ -L "$(ROOT)/compile_commands.json" ]; then rm -- "$(ROOT)/compile_commands.json"; fi
