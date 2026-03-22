# My Telegram Bot

A simple Telegram bot written in **Rust** using [teloxide](https://github.com/teloxide/teloxide) (v0.17).

## Features

- Plain text echoing
- Classic commands (`/start`, `/help`, `/echo`)
- URL decoding (`/urldecode`)
- Base64 encode/decode (`/textbase64encode`, `/textbase64decode`)
- Random number generator (`/rng`)
- Secure random password generation (`/password`)
- Mathematical expression evaluation via `bc` (`/bc`)

## Prerequisites

- Rust 1.90+ (with edition 2024 support)
- `bc` command-line calculator installed  
  - On Termux: `pkg install bc`
  - On Debian/Ubuntu: `sudo apt install bc`
  - On macOS: `brew install bc`

## Installation

```bash
# 1. Clone the repository
git clone https://github.com/CodeIter/my_telegram_bot_rustlang.git
cd my_telegram_bot

# 2. Copy example env file
cp .env.example .env

# 3. Edit .env and add your token
#    TELOXIDE_TOKEN=123456:AAFxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 4. Build & run
cargo run

# 5. Or build release
cargo build --release
```

## Development

```bash
# Watch mode (recommended during development)
cargo watch -x run

# Or release build
cargo run --release
```

## License

[MIT License](LICENSE).

