FROM rust:1.83-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    cmake \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p vortex-indexer

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/vortex-indexer /usr/local/bin/

CMD ["vortex-indexer"]
