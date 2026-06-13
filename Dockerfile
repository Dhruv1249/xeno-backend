# backend/Dockerfile
#
# Multi-stage Dockerfile for containerizing the Rust campaign channel simulator.
#
# Stage 1: Build the release binary
# rust:1.85-alpine is the minimum version with edition2024 support (stabilized in 1.85).
FROM rust:1.89-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app

# Cache dependency compilation separately from source changes.
# Docker layer cache means `cargo build` only re-runs full compile when Cargo.toml changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
# Touch main.rs so Cargo detects the change and relinks with real source.
RUN touch src/main.rs && cargo build --release

# Stage 2: Minimal runtime image
FROM alpine:3.19
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/xeno-backend /usr/local/bin/
EXPOSE 8080
CMD ["xeno-backend"]
