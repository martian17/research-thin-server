# --- Stage 1: Build Environment ---
FROM rust:1.95-slim AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# --- Stage 2: Minimal Runtime Environment ---
FROM debian:13-slim
WORKDIR /app

# Install basic runtime dependencies (like SSL/TLS certificates for network calls)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary and rename it explicitly to your required path
COPY --from=builder /usr/src/app/target/release/research-thin-server /app/prod_binary

RUN chmod +x /app/prod_binary

# Execute the application
ENTRYPOINT ["/app/prod_binary"]
