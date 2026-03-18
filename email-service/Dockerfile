FROM rust:1.92 AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY email-service/Cargo.toml ./email-service/
# Copy Cargo.lock only if it exists
RUN if [ -f email-service/Cargo.lock ]; then \
        cp email-service/Cargo.lock ./email-service/; \
    fi
COPY shared-auth/Cargo.toml ./shared-auth/

# Create dummy source files to build dependencies
WORKDIR /app/email-service
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p ../shared-auth/src && \
    echo "// dummy" > ../shared-auth/src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY email-service/src ./src
# Copy .sqlx only if it exists
RUN if [ -d email-service/.sqlx ]; then \
        cp -r email-service/.sqlx ./.sqlx; \
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

COPY --from=builder /app/email-service/target/release/email-service /usr/local/bin/

RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

CMD ["email-service"]