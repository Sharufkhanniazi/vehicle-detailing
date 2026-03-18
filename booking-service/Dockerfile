FROM rust:1.92 AS builder

WORKDIR /app

# Install build dependencies including protoc and cmake for gRPC and rdkafka
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY booking-service/Cargo.toml ./booking-service/
# Copy Cargo.lock only if it exists
RUN if [ -f booking-service/Cargo.lock ]; then \
        cp booking-service/Cargo.lock ./booking-service/; \
    fi
COPY shared-auth/Cargo.toml ./shared-auth/

# Create proto directory
RUN mkdir -p /app/proto

# Create dummy source files to build dependencies
WORKDIR /app/booking-service
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p proto && \
    echo 'syntax = "proto3"; package pricing;' > /app/proto/pricing.proto && \
    mkdir -p ../shared-auth/src && \
    echo "// dummy" > ../shared-auth/src/lib.rs && \
    echo 'fn main() -> Result<(), Box<dyn std::error::Error>> { tonic_build::compile_protos("../proto/pricing.proto")?; Ok(()) }' > build.rs && \
    cargo build --release && \
    rm -rf src proto build.rs

# Copy the actual source code and proto files
COPY booking-service/build.rs ./build.rs
COPY booking-service/src ./src
COPY booking-service/.sqlx ./.sqlx
COPY proto /app/proto
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

COPY --from=builder /app/booking-service/target/release/booking-service /usr/local/bin/

RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

CMD ["booking-service"]