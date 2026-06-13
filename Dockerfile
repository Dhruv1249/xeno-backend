# backend/Dockerfile
#
# Multi-stage Dockerfile for containerizing the Rust campaign channel simulator.
#
# Stage 1: Build the release binary
FROM rust:1.78-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 2: Minimal runtime image
FROM alpine:3.19
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/xeno-backend /usr/local/bin/
EXPOSE 8080
CMD ["xeno-backend"]
