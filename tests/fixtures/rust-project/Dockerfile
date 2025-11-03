# Test fixture Dockerfile for raibid-ci
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy source
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/libraibid_test_fixture.rlib ./

# Metadata
LABEL org.opencontainers.image.title="raibid-test-fixture"
LABEL org.opencontainers.image.description="Test fixture for raibid-ci E2E testing"

CMD ["echo", "Test fixture built successfully"]
