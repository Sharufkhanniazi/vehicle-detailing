FROM rust:1.92 AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY notification-service/Cargo.toml ./notification-service/
# Copy Cargo.lock only if it exists
RUN if [ -f notification-service/Cargo.lock ]; then \
        cp notification-service/Cargo.lock ./notification-service/; \
    fi
COPY shared-auth/Cargo.toml ./shared-auth/

# Create dummy source files to build dependencies
WORKDIR /app/notification-service
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p ../shared-auth/src && \
    echo "// dummy" > ../shared-auth/src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code and service account key
COPY notification-service/src ./src
# Copy .sqlx only if it exists
RUN if [ -d notification-service/.sqlx ]; then \
        cp -r notification-service/.sqlx ./.sqlx; \
    fi
# Copy service account key only if it exists
RUN if [ -f notification-service/vehicle_detailing.json ]; then \
        cp notification-service/vehicle_detailing.json ./vehicle_detailing.json; \
    fi
COPY shared-auth/src ../shared-auth/src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/notification-service/target/release/notification-service /usr/local/bin/
# Copy service account key if it exists in builder
RUN if [ -f /app/notification-service/vehicle_detailing.json ]; then \
        cp /app/notification-service/vehicle_detailing.json ./; \
    fi

RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 3003

CMD ["notification-service"]