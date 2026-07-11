FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_VERSION=1.96.1

# -----------------------------------------------------------------------------
# System packages
# -----------------------------------------------------------------------------

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
        unzip \
        zip \
        less \
        vim \
        ca-certificates \
        python3 \
        python3-pip \
        xz-utils \
        openjdk-25-jdk && \
    rm -rf /var/lib/apt/lists/*

# -----------------------------------------------------------------------------
# Rust
# -----------------------------------------------------------------------------

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install ${RUST_VERSION} && \
    rustup default ${RUST_VERSION} && \
    rustup component add rustfmt clippy

RUN cargo install cargo-watch

# JDK 25 is installed via apt-get above
ENV JAVA_HOME=/usr/lib/jvm/java-25-openjdk-arm64
ENV PATH="${JAVA_HOME}/bin:${PATH}"

# -----------------------------------------------------------------------------
# Workspace
# -----------------------------------------------------------------------------

WORKDIR /workspace

CMD ["/bin/bash"]

