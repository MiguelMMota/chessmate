# Multi-stage build for ChessMate server

# Build stage
FROM rust:1.86 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build for release (without Godot feature for headless server)
RUN cargo build --bin chessmate-server --release --no-default-features

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/chessmate-server /usr/local/bin/chessmate-server

# Copy migrations for database setup
COPY --from=builder /app/migrations ./migrations

# Expose WebSocket port
EXPOSE 3000

# Set default environment variables
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/chessmate
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the server
CMD ["chessmate-server"]
