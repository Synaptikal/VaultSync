# Build Stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/vaultsync

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project to cache dependencies
RUN cargo new --bin app
WORKDIR /usr/src/vaultsync/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (this layer will be cached)
RUN cargo build --release --locked

# Remove the dummy source
RUN rm src/*.rs

# Copy actual source code
COPY src ./src

# Build the application
# We touch main.rs to ensure it recompiles
RUN touch src/main.rs
RUN cargo build --release --locked

# Runtime Stage
FROM debian:bookworm-slim

# Create a non-root user
RUN useradd -ms /bin/bash vaultsync

WORKDIR /app

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/vaultsync/app/target/release/vaultsync /app/vaultsync

# Create data directory and set permissions
RUN mkdir -p /app/data \
    && chown -R vaultsync:vaultsync /app

# Switch to non-root user
USER vaultsync

# Expose the API port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///app/data/vaultsync.db
ENV HOST=0.0.0.0
ENV PORT=3000

# Run the application
CMD ["./vaultsync"]
