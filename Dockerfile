# Multi-stage Dockerfile for Rust API
FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates

# Build the application
RUN cargo build --release

# Runtime stage with minimal image
FROM debian:bookworm-slim

# Install runtime dependencies including curl for health check
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false appuser

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/rust-api /app/rust-api

# Copy templates to runtime stage (needed for template loading)
COPY --from=builder /app/templates ./templates

# Change ownership to non-root user
RUN chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Run the application
CMD ["./rust-api"] 