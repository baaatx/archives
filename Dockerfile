# Archives Production Dockerfile
# Multi-stage build for minimal final image

# ============================================================================
# Stage 1: Build
# ============================================================================
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/archives-common/Cargo.toml crates/archives-common/
COPY crates/archives-api/Cargo.toml crates/archives-api/
COPY crates/archives-mcp/Cargo.toml crates/archives-mcp/
COPY crates/archives-cli/Cargo.toml crates/archives-cli/

# Create dummy source files to build dependencies
RUN mkdir -p crates/archives-common/src && echo "pub fn dummy() {}" > crates/archives-common/src/lib.rs
RUN mkdir -p crates/archives-api/src && echo "fn main() {}" > crates/archives-api/src/main.rs
RUN mkdir -p crates/archives-mcp/src && echo "fn main() {}" > crates/archives-mcp/src/main.rs
RUN mkdir -p crates/archives-cli/src && echo "fn main() {}" > crates/archives-cli/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release --workspace && rm -rf crates/*/src

# Copy actual source code
COPY crates/ crates/

# Touch main files to trigger rebuild
RUN touch crates/archives-common/src/lib.rs \
    crates/archives-api/src/main.rs \
    crates/archives-mcp/src/main.rs \
    crates/archives-cli/src/main.rs

# Build actual binaries
RUN cargo build --release --workspace

# ============================================================================
# Stage 2: API Runtime
# ============================================================================
FROM debian:bookworm-slim AS api

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r archives && useradd -r -g archives archives

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/archives-api /app/archives-api

# Set ownership
RUN chown -R archives:archives /app

USER archives

# Default environment variables
ENV RUST_LOG=archives_api=info
ENV ARCHIVES__API__HOST=0.0.0.0
ENV ARCHIVES__API__PORT=8080

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/archives-api"]

# ============================================================================
# Stage 3: MCP Runtime
# ============================================================================
FROM debian:bookworm-slim AS mcp

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r archives && useradd -r -g archives archives

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/archives-mcp /app/archives-mcp

# Set ownership
RUN chown -R archives:archives /app

USER archives

# Default environment variables
ENV RUST_LOG=archives_mcp=info
ENV ARCHIVES__MCP__HOST=0.0.0.0
ENV ARCHIVES__MCP__PORT=8081

EXPOSE 8081

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

ENTRYPOINT ["/app/archives-mcp"]

# ============================================================================
# Stage 4: CLI
# ============================================================================
FROM debian:bookworm-slim AS cli

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r archives && useradd -r -g archives archives

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/archives /app/archives

# Set ownership
RUN chown -R archives:archives /app

USER archives

ENTRYPOINT ["/app/archives"]
