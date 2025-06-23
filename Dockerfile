# syntax=docker/dockerfile:1.4
FROM ubuntu:24.04

LABEL maintainer="gengi"

# --- Build args passed from compiler.rs ---
ARG PROFILE=linux_default
ARG TARGET=x86_64-unknown-linux-musl
ARG RELEASE=false
ARG FEATURES=
ARG STRIP=false

ENV PROFILE=${PROFILE}
ENV TARGET=${TARGET}
ENV RELEASE=${RELEASE}
ENV FEATURES=${FEATURES}
ENV STRIP=${STRIP}

WORKDIR /app

# --- Install Dependencies ---
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    file \
    sudo \
    g++-mingw-w64-x86-64 \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    musl-tools \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# --- Install Rust with minimal profile ---
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

# --- Add all common cross-compilation targets ---
RUN rustup target add \
    x86_64-unknown-linux-musl \
    aarch64-unknown-linux-gnu \
    x86_64-pc-windows-gnu

# -- copy project files --
COPY ./build_profiles /app/build_profiles
COPY ./src /app/src/
COPY ./Cargo.toml /app

# --- Build binary dynamically ---
CMD bash -c '\
    set -e; \
    echo "[*] Using profile: $PROFILE"; \
    echo "[*] Target: $TARGET"; \
    echo "[*] Release: $RELEASE"; \
    echo "[*] Features: $FEATURES"; \
    echo "[*] Strip: $STRIP"; \
    BUILD_CMD="cargo build --target $TARGET"; \
    if [ "$RELEASE" = "true" ]; then BUILD_CMD="$BUILD_CMD --release"; fi; \
    if [ -n "$FEATURES" ]; then BUILD_CMD="$BUILD_CMD --features $FEATURES"; fi; \
    echo "[*] Running build: $BUILD_CMD"; \
    eval $BUILD_CMD; \
    BIN_DIR="target/$TARGET/$( [ "$RELEASE" = "true" ] && echo release || echo debug )"; \
    mkdir -p /output; \
    for bin in $BIN_DIR/*; do \
        if [ -x "$bin" ] && [ -f "$bin" ]; then \
            [ "$STRIP" = "true" ] && echo "[*] Stripping $bin" && strip "$bin"; \
            echo "[*] Copying $bin to /output"; \
            cp "$bin" /output/; \
        fi; \
    done; \
    echo "[+] Done." \
'