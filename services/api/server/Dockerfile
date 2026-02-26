ARG RUST_VERSION=1.92.0

# Use official Rust image - Blacksmith will cache this layer efficiently
FROM rust:${RUST_VERSION}-slim-bookworm AS chef

# Install cargo-chef in a separate cached layer
RUN cargo install cargo-chef --locked

# Install system dependencies - highly cacheable layer
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    clang \
    lld \
    ca-certificates \
    curl \
    make \
    autoconf \
    automake \
    libtool \
    zlib1g-dev \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

# ============================================
# Planner stage - generates dependency recipe
# ============================================
FROM chef AS planner

# Copy the entire workspace for recipe generation
# This needs to be done because cargo chef needs the full workspace structure
COPY . .

# Generate recipe - this layer changes only when dependencies change
RUN cargo chef prepare --recipe-path recipe.json

# ============================================
# Dependencies stage - builds only dependencies
# ============================================
FROM chef AS dependencies

# Copy the recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies with Blacksmith's enhanced caching
# This stage will be heavily cached by Blacksmith's NVMe layer cache
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# ============================================
# Builder stage - builds the application
# ============================================
FROM dependencies AS builder

# Copy source code
COPY . .

# Build application - dependency artifacts are already present
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin wire-api && \
    strip target/release/wire-api && \
    cp target/release/wire-api /usr/local/bin/wire-api

# ============================================
# Development stage - for local development with hot reload
# ============================================
FROM chef AS development

# Install cargo-watch for hot reloading
RUN cargo install cargo-watch --locked

# Copy the recipe and cook dependencies first (for better caching)
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo chef cook --recipe-path recipe.json

# Copy source code
COPY . .

# Expose port
EXPOSE 8080

# This will be the entrypoint for dev with hot reload
CMD ["cargo", "watch", "--poll", \
    "-i", "data/*", \
    "-i", "target/*", \
    "-i", "node_modules/*", \
    "-i", "*.log", \
    "-i", ".git/*", \
    "-x", "run --bin wire-api"]


# ============================================
# Runtime stage - minimal final image
# ============================================
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies - cached layer
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user - cached layer
RUN groupadd -r appuser && \
    useradd -r -g appuser -d /home/appuser -s /bin/bash appuser && \
    mkdir -p /home/appuser && \
    chown -R appuser:appuser /home/appuser

# Copy binary from builder
COPY --from=builder /usr/local/bin/wire-api /usr/local/bin/wire-api
RUN chmod +x /usr/local/bin/wire-api

# Switch to non-root user
USER appuser
WORKDIR /home/appuser

# Expose port for the API
EXPOSE 8080

# Start application
CMD ["wire-api"]
