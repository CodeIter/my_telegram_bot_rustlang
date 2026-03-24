FROM rust:1.85-alpine AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release --bin my_telegram_bot

FROM alpine:3.21

# Install runtime dependencies (yt-dlp, bc, ffmpeg)
RUN apk add --no-cache \
    ca-certificates \
    ffmpeg \
    bc \
    python3 \
    py3-pip \
    && python3 -m venv /venv \
    && /venv/bin/pip install --no-cache-dir yt-dlp \
    && ln -s /venv/bin/yt-dlp /usr/local/bin/yt-dlp

# Copy the compiled binary
COPY --from=builder /app/target/release/my_telegram_bot /usr/local/bin/my_telegram_bot

# Run as non-root user
RUN adduser -D -u 1001 botuser
USER botuser

WORKDIR /app

CMD ["my_telegram_bot"]
