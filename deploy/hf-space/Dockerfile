# Stage 1: Build static release binary
FROM rust:1.80-alpine AS builder
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy Cargo configuration and dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Compile the showcase binary in release mode
RUN cargo build --release --example showcase

# Stage 2: Minimal Hugging Face Spaces runtime image
FROM alpine:3.20 AS runner
WORKDIR /app

# Hugging Face Spaces runs as user 1000
RUN adduser -D -u 1000 appuser && \
    mkdir -p /app && \
    chown -R appuser:appuser /app

# Copy the compiled binary
COPY --from=builder /app/target/release/examples/showcase /app/showcase
RUN chmod +x /app/showcase

USER appuser

# Hugging Face default application port
EXPOSE 7860
ENV PORT=7860
ENV HOST=0.0.0.0

CMD ["/app/showcase"]
