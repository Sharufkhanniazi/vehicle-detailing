# Dockerfile (located in auth-service/Dockerfile)
FROM rust:1.92 AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY auth-service/Cargo.toml ./auth-service/
# Copy Cargo.lock only if it exists
RUN if [ -f auth-service/Cargo.lock ]; then \
        cp auth-service/Cargo.lock ./auth-service/; \
    fi
COPY shared-auth/Cargo.toml ./shared-auth/

# Create dummy source files to build dependencies
WORKDIR /app/auth-service
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p ../shared-auth/src && \
    echo "// dummy" > ../shared-auth/src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY auth-service/src ./src
# Copy .sqlx only if it exists
RUN if [ -d auth-service/.sqlx ]; then \
        cp -r auth-service/.sqlx ./.sqlx; \
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

COPY --from=builder /app/auth-service/target/release/auth-service /usr/local/bin/

RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

CMD ["auth-service"]