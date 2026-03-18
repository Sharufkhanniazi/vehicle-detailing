FROM rust:1.92 AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY tracking-service/Cargo.toml ./tracking-service/
# Copy Cargo.lock only if it exists
RUN if [ -f tracking-service/Cargo.lock ]; then \
        cp tracking-service/Cargo.lock ./tracking-service/; \
    fi
COPY shared-auth/Cargo.toml ./shared-auth/

# Create dummy source files to build dependencies
WORKDIR /app/tracking-service
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p ../shared-auth/src && \
    echo "// dummy" > ../shared-auth/src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY tracking-service/src ./src
# Copy .sqlx only if it exists
RUN if [ -d tracking-service/.sqlx ]; then \
        cp -r tracking-service/.sqlx ./.sqlx; \
    fi
COPY shared-auth/src ../shared-auth/src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/tracking-service/target/release/tracking-service /usr/local/bin/

RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 3002

CMD ["tracking-service"]