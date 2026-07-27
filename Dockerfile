FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_VERSION=1.96.1

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        clang \
        llvm \
        lldb \
        lld \
        cmake \
        ninja-build \
        pkg-config \
        libclang-dev \
        git \
        curl \
        wget \
        ca-certificates \
        python3 \
        xz-utils \
        openjdk-21-jdk-headless && \
    rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup component add rustfmt clippy && \
    cargo install cargo-watch

# Resolve JAVA_HOME for both amd64 and arm64 package layouts.
RUN JH="$(dirname "$(dirname "$(readlink -f "$(command -v javac)")")")" && \
    printf '%s\n' "export JAVA_HOME=${JH}" "export PATH=\"\${JAVA_HOME}/bin:\${PATH}\"" \
      > /etc/profile.d/java.sh

ENV PATH="/root/.cargo/bin:${PATH}"

# Classfiles: `make classes` uses javac --release 8.

WORKDIR /workspace

CMD ["/bin/bash"]
