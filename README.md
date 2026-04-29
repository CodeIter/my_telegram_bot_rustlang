# My Telegram Bot

**A powerful, lightweight, and fully-featured Telegram bot written in Rust** 🦀

Built with [teloxide](https://github.com/teloxide/teloxide) v0.17, this bot
demonstrates modern Rust practices including async/await, strong error
handling, clean architecture, and multi-stage Docker builds.

Whether you're looking for a fun utility bot or a solid foundation to build
your own Telegram bot, this project combines simplicity with real-world
functionality.

This bot is perfect for learning Rust + Telegram bot development or as a
starting point for your own advanced bot.

## Features

### 🤖 AI Chat
- **Free-text chat** — any message without a command is automatically answered by GLM-5 AI with full conversation context
- **`/glm5ai <prompt>`** — ask GLM-5 AI (fast, no reasoning)
- **`/glm5aireasoning <prompt>`** — ask GLM-5 AI with step-by-step reasoning enabled
- **`/gemini2 <prompt>`** — ask Google Gemini 2.5 Flash
- **`/gemini3 <prompt>`** — ask Google Gemini 3 Flash Preview
- **Conversation history** — all AI commands include previous messages as context for coherent multi-turn conversations
- **`/clear`** — delete your entire conversation history and reset the AI context

### 🛠️ Utilities
- **`/bc <expression>`** — evaluate math expressions using `bc` (e.g. `/bc sqrt(16)`, `/bc 2+2*3`)
- **`/rng <min> <max>`** — generate a random number in range (min > 0)
- **`/password <length>`** — generate a secure random password (2–128 characters)
- **`/urldecode <encoded>`** — decode a percent-encoded URL string
- **`/textbase64encode <text>`** — encode text to Base64
- **`/textbase64decode <text>`** — decode a Base64 string
- **`/textqr <text>`** — generate and send a QR code image for any text

### 📥 Media Download
- **`/ytdl <url>`** — download a video with `yt-dlp` and send it directly in chat
- **`/ytdlmp3 <url>`** — download audio with `yt-dlp` and send it as an MP3 file

### 📨 Media Echo
- Automatically echoes back **stickers, animations, videos, photos, voice messages,** and **documents** sent to the bot

### 💾 Persistence
- All conversations (user messages and bot replies) are stored in a local **SQLite** database
- User profiles are upserted on every interaction — no manual registration needed

## Prerequisites

**Local development**  
- Rust 1.90+ (edition 2024)  
- `bc`, `yt-dlp`, `ffmpeg`

**Docker (recommended – zero local dependencies)**  
- Docker + Docker Compose

## Installation

```bash
git clone https://github.com/CodeIter/my_telegram_bot_rustlang
cd my_telegram_bot_rustlang

cp .env.example .env
# Edit .env and add your tokens:

```

### Option 1: Local (Cargo)

```bash
cargo run
```

### Option 2: Docker

```bash
docker compose up --build -d
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
