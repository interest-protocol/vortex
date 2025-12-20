FROM rustlang/rust:nightly AS builder

WORKDIR work

RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config curl cmake clang ca-certificates
ENV PATH="/root/.cargo/bin:${PATH}"

COPY indexer/Cargo.lock indexer/Cargo.toml ./
COPY indexer/crates/ ./crates/

RUN cargo build --release --bin vortex-indexer --config net.git-fetch-with-cli=true

FROM debian:sid-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /work/target/release/vortex-indexer /usr/local/bin/

CMD ["vortex-indexer"]
