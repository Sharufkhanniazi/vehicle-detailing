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
COPY pricing-service/Cargo.toml ./pricing-service/
# Copy Cargo.lock only if it exists
RUN if [ -f pricing-service/Cargo.lock ]; then \
        cp pricing-service/Cargo.lock ./pricing-service/; \
    fi

# Create proto directory
RUN mkdir -p /app/proto

# Create dummy source files to build dependencies
WORKDIR /app/pricing-service
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p proto && \
    echo 'syntax = "proto3"; package pricing;' > /app/proto/pricing.proto && \
    echo 'fn main() -> Result<(), Box<dyn std::error::Error>> { tonic_build::compile_protos("../proto/pricing.proto")?; Ok(()) }' > build.rs && \
    cargo build --release && \
    rm -rf src proto build.rs

# Copy the actual source code and proto files
COPY pricing-service/build.rs ./build.rs
COPY pricing-service/src ./src
COPY proto /app/proto

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/pricing-service/target/release/pricing-service /usr/local/bin/

RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 50051

CMD ["pricing-service"]