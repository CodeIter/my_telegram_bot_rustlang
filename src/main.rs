use dotenvy::dotenv;
use std::sync::Arc;
use teloxide::{
    RequestError,
    dispatching::{UpdateHandler, dialogue::InMemStorage},
    filter_command,
    prelude::*,
    types::Update,
    utils::command::BotCommands,
}; // ← for default_handler

use base64::{Engine as _, engine::general_purpose};
use percent_encoding::percent_decode_str;
use rand::Rng;

use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioProcessCommand;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("🚀 Starting Telegram bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<()>::new()])
        .default_handler(|upd: Arc<Update>| async move {
            log::warn!("Unhandled update: {upd:#?}");
        })
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<RequestError> {
    dptree::entry().branch(
        Update::filter_message()
            .branch(filter_command::<Command, _>().endpoint(
                |bot: Bot, msg: Message, cmd: Command| async move {
                    command_handler(bot, msg, cmd).await
                },
            ))
            .branch(Update::filter_message().endpoint(echo_text_handler)),
    )
}

// ── Commands ────────────────────────────────────────────────────────
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Say hello!")]
    Start,
    #[command(description = "Echo any text (but we also echo without command)")]
    Echo(String),

    #[command(description = "/urldecode <encoded> → decode URL")]
    UrlDecode(String),

    #[command(description = "/textbase64encode <text> → encode to base64")]
    TextBase64Encode(String),

    #[command(description = "/textbase64decode <text> → decode base64")]
    TextBase64Decode(String),

    #[command(
        description = "/rng <min> <max> → random number (min > 0)",
        parse_with = "split"
    )]
    Rng(u32, u32),

    #[command(description = "/password <length> → generate password (>1)")]
    Password(u32),

    #[command(description = "/bc <expression> → calculate with bc (e.g. /bc 2+2*3)")]
    Bc(String),
}

async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match cmd {
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions().to_string())
                .await?;
        }
        Command::Start => {
            bot.send_message(
                chat_id,
                "👋 Hello! I'm your Rust 🦀 bot powered by teloxide!\nJust type anything → I will echo it!\nUse /help for commands list",
            )
            .await?;
        }
        Command::Echo(text) => {
            bot.send_message(chat_id, format!("📢 : {text}")).await?;
        }

        Command::UrlDecode(encoded) => {
            let decoded = percent_decode_str(&encoded).decode_utf8_lossy().to_string();
            bot.send_message(chat_id, format!("🔓 Decoded URL:\n{}", decoded))
                .await?;
        }

        Command::TextBase64Encode(text) => {
            let encoded = general_purpose::STANDARD.encode(text.as_bytes());
            bot.send_message(chat_id, format!("🔼 Base64 encoded:\n{}", encoded))
                .await?;
        }

        Command::TextBase64Decode(encoded) => match general_purpose::STANDARD.decode(&encoded) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => {
                    bot.send_message(chat_id, format!("🔽 Base64 decoded:\n{}", decoded))
                        .await?;
                }
                Err(_) => {
                    bot.send_message(chat_id, "❌ Not valid UTF-8").await?;
                }
            },
            Err(_) => {
                bot.send_message(chat_id, "❌ Invalid Base64").await?;
            }
        },

        Command::Rng(min, max) => {
            if min == 0 || max == 0 || min > max {
                bot.send_message(chat_id, "❌ Use: /rng 1 100 (min > 0, max > min)")
                    .await?;
            } else {
                let num = rand::thread_rng().gen_range(min..=max);
                bot.send_message(chat_id, format!("🎲 Random number: **{}**", num))
                    .await?;
            }
        }

        Command::Password(len) => {
            if !(2..=128).contains(&len) {
                bot.send_message(chat_id, "❌ Length must be 2–128").await?;
            } else {
                let pw = {
                    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=".chars().collect();
                    let mut rng = rand::thread_rng();
                    (0..len)
                        .map(|_| chars[rng.gen_range(0..chars.len())])
                        .collect::<String>()
                };

                bot.send_message(chat_id, format!("🔑 Password ({} chars):\n`{}`", len, pw))
                    .await?;
            }
        }

        Command::Bc(expr) => {
            if expr.trim().is_empty() {
                bot.send_message(chat_id, "❌ Usage: /bc 2+2*3 or /bc sqrt(16)")
                    .await?;
            } else {
                match run_bc(&expr).await {
                    Ok(result) => {
                        bot.send_message(chat_id, format!("📊 bc result:\n`{}`", result.trim()))
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("❌ bc error: {}", e))
                            .await?;
                    }
                }
            }
        }
    }
    Ok(())
}

// ── Echo ANY plain text (no / needed) ───────────────────────────────
async fn echo_text_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            return Ok(()); // already handled by command branch
        }
        bot.send_message(msg.chat.id, format!("📢 : {text}"))
            .await?;
    }
    Ok(())
}

async fn run_bc(expr: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = TokioProcessCommand::new("bc")
        .arg("-l") // enable math functions (sqrt, sin, etc.)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(expr.as_bytes()).await?;
        stdin.write_all(b"\n").await?; // bc needs a newline
        stdin.flush().await?;
    }

    let output = child.wait_with_output().await?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "bc exited with code {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip() {
        let text = "Hello 🦀 Telegram Bot! 123";
        let encoded = general_purpose::STANDARD.encode(text.as_bytes());
        let decoded_bytes = general_purpose::STANDARD.decode(&encoded).unwrap();
        let decoded = String::from_utf8(decoded_bytes).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn url_decode_works() {
        let encoded = "hello%20world%21%40";
        let decoded = percent_decode_str(encoded).decode_utf8_lossy().into_owned();
        assert_eq!(decoded, "hello world!@");
    }

    #[test]
    fn rng_always_in_range() {
        let min = 10u32;
        let max = 20u32;
        for _ in 0..50 {
            let n = rand::thread_rng().gen_range(min..=max);
            assert!(n >= min && n <= max);
        }
    }

    #[test]
    fn password_correct_length_and_charset() {
        let len = 15u32;
        let chars: Vec<char> =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-="
                .chars()
                .collect();

        let pw = {
            let mut rng = rand::thread_rng();
            (0..len)
                .map(|_| chars[rng.gen_range(0..chars.len())])
                .collect::<String>()
        };

        assert_eq!(pw.len(), len as usize);
        assert!(pw.chars().all(|c| chars.contains(&c)));
    }

    #[tokio::test]
    async fn bc_calculator_basic() {
        let res = run_bc("2 + 2 * 3").await.unwrap();
        assert_eq!(res.trim(), "8");
    }

    #[tokio::test]
    async fn bc_calculator_with_sqrt() {
        let res = run_bc("scale=0; sqrt(16)").await.unwrap();
        assert_eq!(res.trim(), "4");
    }

    #[tokio::test]
    async fn bc_error_handling() {
        let res = run_bc("syntax error!").await;
        assert!(res.is_err()); // bc should return non-zero
    }
}
