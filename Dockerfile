# Multi-stage build - Build stage
FROM rust:latest as builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
# Create dummy main.rs for pre-compiling dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Build dependencies (cache layer)
RUN cargo build --release && rm -rf src
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM bitnami/minideb:latest
WORKDIR /app
RUN install_packages ca-certificates curl && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sysinfo-api /usr/local/bin/sysinfo-api
RUN useradd -m -u 1000 apiuser && chown -R apiuser:apiuser /app
USER apiuser
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f -u admin:password123 http://localhost:8080/api/v1/health || exit 1
CMD ["sysinfo-api"]