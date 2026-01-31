# Build stage
FROM docker.io/rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/

# Create dummy src to build dependencies
RUN mkdir -p backend/src && \
    echo "fn main() {}" > backend/src/main.rs

# Build dependencies only
RUN cargo build --release -p babel && \
    rm -rf backend/src

# Copy actual source code
COPY backend/src ./backend/src

# Touch source files to ensure rebuild
RUN touch backend/src/main.rs

# Build the application
RUN cargo build --release -p babel

# Runtime stage
FROM docker.io/debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/babel /app/babel

# Copy config files if needed
COPY backend/filter_config.json ./

# Expose the default port
EXPOSE 3000

# Run the binary
CMD ["./babel"]
