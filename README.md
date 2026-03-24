# My Telegram Bot

**A powerful, lightweight, and fully-featured Telegram bot written in Rust** 🦀

Built with [teloxide](https://github.com/teloxide/teloxide) v0.17, this bot
demonstrates modern Rust practices including async/await, strong error
handling, clean architecture, and multi-stage Docker builds.

Whether you're looking for a fun utility bot or a solid foundation to build
your own Telegram bot, this project combines simplicity with real-world
functionality.

### What makes it special?

- **Zero external dependencies when running in Docker** — everything (including `yt-dlp` and `bc`) is bundled.
- **Beautiful inline command menu** — just type `/` and see all available commands with descriptions.
- **Rich media support** — echoes stickers, GIFs, photos, videos, voice messages, and documents back to the user.
- **Practical utilities** in one place: encoding, decoding, randomness, calculations, YouTube downloads, and QR code generation.
- **Production-ready** Docker setup with security best practices (non-root user, minimal Alpine base).

This bot is perfect for learning Rust + Telegram bot development or as a
starting point for your own advanced bot.

## Features

- Plain text echoing (plus full media echo: stickers, GIFs, photos, videos, voice messages, documents)
- Classic commands (`/start`, `/help`)
- `/echo <text>`
- URL decoding (`/urldecode <encoded>`)
- Base64 encode/decode (`/textbase64encode <text>`, `/textbase64decode <text>`)
- Random number generator (`/rng <min> <max>`)
- Secure random password generation (`/password <length>`)
- Mathematical expression evaluation via `bc` (`/bc <expression>`)
- YouTube video download (`/ytdl <url>`) → sends MP4
- YouTube audio download (`/ytdlmp3 <url>`) → sends MP3
- QR code generation (`/textqr <text>`) → sends PNG image

All commands are automatically registered with Telegram and appear in the **/** menu when you type `/`.

## Prerequisites

**Local development**  
- Rust 1.90+ (edition 2024)  
- `bc`, `yt-dlp`, `ffmpeg`

**Docker (recommended – zero local dependencies)**  
- Docker + Docker Compose

## Installation

### Option 1: Local (Cargo)

```bash
git clone https://github.com/CodeIter/my_telegram_bot_rustlang.git
cd my_telegram_bot

cp .env.example .env
# Edit .env and add your token:
# TELOXIDE_TOKEN=123456:AAFxxxxxxxxxxxxxxxxxxxxxxxxxxxx

cargo run
```

### Option 2: Docker (recommended)

```bash
git clone https://github.com/CodeIter/my_telegram_bot_rustlang.git
cd my_telegram_bot

cp .env.example .env
# Edit .env with your TELOXIDE_TOKEN

docker compose up --build -d
```

**Useful commands:**
```bash
docker compose logs -f          # live logs
docker compose restart          # restart bot
docker compose down             # stop & remove container
```

## Development

```bash
# Watch mode (recommended)
cargo watch -x run

# Or release build
cargo run --release
```

## License

[MIT License](LICENSE).
