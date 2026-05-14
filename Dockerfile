# Stage 1: Build
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Run
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/research-site .
# Create content dir to mount into
RUN mkdir ./content 
EXPOSE 8080
CMD ["./research-site"]
