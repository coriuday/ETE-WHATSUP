# ─────────────────────────────────────────────
# Rust Axum Backend API Dockerfile
# ─────────────────────────────────────────────

FROM rust:slim-bookworm AS builder

WORKDIR /app

# Install dependencies required for openssl
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo configs
COPY Cargo.toml Cargo.lock ./

# Create dummy src file to build dependencies and cache them
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# Copy actual source files
COPY src/ ./src/

# Rebuild actual application
RUN cargo build --release

# Runner stage
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install runtime dependencies (like OpenSSL and CA certificates for HTTPS)
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/api /app/api
COPY .env.example /app/.env.example

EXPOSE 8080

CMD ["/app/api"]
