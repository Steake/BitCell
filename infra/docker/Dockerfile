# Multi-stage build for BitCell node
FROM rust:1.82-bookworm as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY crates ./crates

# Build the node binary in release mode
RUN cargo build --release --bin bitcell-node

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 bitcell && \
    mkdir -p /data/bitcell && \
    chown -R bitcell:bitcell /data/bitcell

# Copy binary from builder
COPY --from=builder /build/target/release/bitcell-node /usr/local/bin/bitcell-node

# Copy startup script
COPY infra/docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

USER bitcell
WORKDIR /home/bitcell

# Expose ports
EXPOSE 9000 8545 9090

ENTRYPOINT ["/entrypoint.sh"]
