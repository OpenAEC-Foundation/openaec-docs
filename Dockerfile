# Stage 1: Build the frontend
FROM node:22-slim AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build the Rust binary
FROM rust:latest AS builder

# Limit parallelism to reduce memory usage on small servers
ENV CARGO_BUILD_JOBS=1

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/docs-server/Cargo.toml crates/docs-server/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p crates/docs-server/src && \
    echo "fn main() {}" > crates/docs-server/src/main.rs

# Copy migrations (needed for sqlx compile-time checks)
COPY migrations/ migrations/

# Build dependencies only (cached layer)
RUN cargo build --release -p docs-server 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/

# Touch source files to invalidate cache, then build
RUN touch crates/docs-server/src/main.rs && \
    cargo build --release -p docs-server

# Stage 3: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -r docs && useradd -r -g docs -d /app docs

WORKDIR /app

COPY --from=builder /app/target/release/docs-server /app/docs-server
COPY --from=frontend-builder /app/frontend/dist /app/static
COPY migrations/ /app/migrations/

RUN mkdir -p /app/data && chown -R docs:docs /app

USER docs

EXPOSE 3000

ENV RUST_LOG=docs_server=info,tower_http=info

CMD ["/app/docs-server"]
