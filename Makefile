# Make coordinates the tools; CMake and Cargo track their own dependencies.
# Run from the repository root, or use make -C /path/to/klover.
.DEFAULT_GOAL := build
override ROOT := $(CURDIR)

PROFILE ?= debug
JOBS ?= 4
CMAKE ?= cmake
CTEST ?= ctest
CARGO ?= cargo
JAVAC ?= javac

ifeq ($(PROFILE),debug)
CMAKE_BUILD_TYPE := Debug
CARGO_PROFILE := dev
else ifeq ($(PROFILE),release)
CMAKE_BUILD_TYPE := Release
CARGO_PROFILE := release
else
$(error PROFILE must be debug or release)
endif

CORE_DIR := $(ROOT)/build/$(PROFILE)/core
CARGO_DIR := $(ROOT)/build/cargo
CLASSES_DIR := $(ROOT)/build/classes
CARGO_FLAGS := --manifest-path "$(ROOT)/rust/Cargo.toml" --locked \
  --profile $(CARGO_PROFILE) --target-dir "$(CARGO_DIR)" --jobs $(JOBS)
# Each tool manages its own JOBS limit, even when Make itself runs with -j.
CARGO_ENV := MAKEFLAGS= MFLAGS= KLOVER_CORE_DIR="$(CORE_DIR)"

.PHONY: build build-core build-rust test test-core test-rust classes clean help

build: build-rust

build-core:
	"$(CMAKE)" -S "$(ROOT)" -B "$(CORE_DIR)" -DCMAKE_BUILD_TYPE=$(CMAKE_BUILD_TYPE) -DBUILD_TESTING=ON
	MAKEFLAGS= MFLAGS= "$(CMAKE)" --build "$(CORE_DIR)" --config $(CMAKE_BUILD_TYPE) --parallel $(JOBS)

# Shared prerequisites run once per Make invocation, including with make -j.
build-rust: build-core
	$(CARGO_ENV) "$(CARGO)" build $(CARGO_FLAGS)

test: test-core test-rust

test-core: build-core
	"$(CTEST)" --test-dir "$(CORE_DIR)" --build-config $(CMAKE_BUILD_TYPE) --output-on-failure --no-tests=error --parallel $(JOBS)

test-rust: build-core
	$(CARGO_ENV) "$(CARGO)" test $(CARGO_FLAGS)

# The Java library is small: compile it together on each explicit request.
# Clear old classes so deleted or renamed sources cannot leave stale output.
classes:
	rm -rf -- "$(CLASSES_DIR)"
	mkdir -p "$(CLASSES_DIR)"
	cd "$(ROOT)" && find java/java.base -type f -name '*.java' -exec printf '"%s"\n' {} \; > "$(CLASSES_DIR)/sources.list"
	"$(JAVAC)" --release 8 -d "$(CLASSES_DIR)" "@$(CLASSES_DIR)/sources.list"

clean:
	rm -rf -- "$(ROOT)/build"

help:
	@printf '%s\n' \
	  'make                         Build the C core and Rust library (debug)' \
	  'make test                    Run C and Rust tests' \
	  'make classes                 Compile the Java library for Java 8' \
	  'make PROFILE=release         Build release libraries' \
	  'make test PROFILE=release    Run release tests' \
	  'make build-core              Build only C and its test executables' \
	  'make build-rust              Build C, then the Rust library' \
	  'make test-core               Build and test only C' \
	  'make test-rust               Build C, then run Rust tests' \
	  'make clean                   Remove all outputs under build/' \
	  '' \
	  'Settings: PROFILE=debug|release, JOBS=4' \
	  'Tool overrides: CMAKE, CTEST, CARGO, JAVAC (executable paths)'
