# Build Stage 1: Create a layer for dependency caching
FROM rust:slim-bookworm AS deps
WORKDIR /app

# Install only necessary build dependencies and cleanup tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Clean up any previous builds that might exist
RUN cargo clean || true && \
    rm -rf /usr/local/cargo/registry/* || true && \
    rm -rf /usr/local/cargo/git/* || true

# Copy only files needed for dependency resolution
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src && \
    # Clean up unnecessary files after dependency build
    rm -rf target/release/.fingerprint/rust-api-* && \
    rm -rf target/release/deps/rust_api-* && \
    rm -rf target/release/rust-api*

# Build Stage 2: Build the actual application
FROM deps AS builder
WORKDIR /app

# Copy source code and templates
COPY src ./src
COPY templates ./templates

# Build with optimization flags and clean up unnecessary files
RUN RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release && \
    strip target/release/rust-api && \
    # Clean up everything except the final binary
    find target/release -type f ! -name 'rust-api' -delete && \
    rm -rf target/release/deps target/release/.fingerprint target/release/build && \
    rm -rf /usr/local/cargo/registry/* && \
    rm -rf /usr/local/cargo/git/*

# Runtime Stage: Create the final minimal image
FROM debian:bookworm-slim AS runtime

# Install only essential runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -r -s /bin/false appuser

WORKDIR /app

# Copy only the necessary files from builder
COPY --from=builder /app/target/release/rust-api /app/rust-api
COPY --from=builder /app/templates ./templates

# Set ownership and permissions
RUN chown -R appuser:appuser /app && \
    chmod +x /app/rust-api

USER appuser

# Configure container
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/ || exit 1

CMD ["./rust-api"]