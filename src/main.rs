use dotenvy::dotenv;
use std::sync::Arc;
use teloxide::{
    RequestError,
    dispatching::{UpdateHandler, dialogue::InMemStorage},
    filter_command,
    prelude::*,
    sugar::request::RequestReplyExt,
    types::{InputFile, ParseMode, Update},
    utils::command::BotCommands,
    utils::markdown::escape,
};

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
    match cmd {
        Command::Help => {
            reply_markdown(bot, msg, Command::descriptions().to_string()).await?;
        }
        Command::Start => {
            reply_markdown(
                bot,
                msg,
                "👋 Hello! I'm your Rust 🦀 bot powered by teloxide!\nJust type anything → I will echo it!\nUse /help for commands list".to_string(),
            )
            .await?;
        }
        Command::Echo(text) => {
            reply_markdown(bot, msg, format!("📢 : {text}")).await?;
        }

        Command::UrlDecode(encoded) => {
            let decoded = percent_decode_str(&encoded).decode_utf8_lossy().to_string();
            reply_markdown(bot, msg, format!("🔓 Decoded URL:\n{}", decoded)).await?;
        }

        Command::TextBase64Encode(text) => {
            let encoded = general_purpose::STANDARD.encode(text.as_bytes());
            reply_markdown(bot, msg, format!("🔼 Base64 encoded:\n{}", encoded)).await?;
        }

        Command::TextBase64Decode(encoded) => match general_purpose::STANDARD.decode(&encoded) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => {
                    reply_markdown(bot, msg, format!("🔽 Base64 decoded:\n{}", decoded)).await?;
                }
                Err(_) => {
                    reply_markdown(bot, msg, "❌ Not valid UTF-8".to_string()).await?;
                }
            },
            Err(_) => {
                reply_markdown(bot, msg, "❌ Invalid Base64".to_string()).await?;
            }
        },

        Command::Rng(min, max) => {
            if min == 0 || max == 0 || min > max {
                reply_markdown(
                    bot,
                    msg,
                    "❌ Use: /rng 1 100 (min > 0, max > min)".to_string(),
                )
                .await?;
            } else {
                let num = rand::thread_rng().gen_range(min..=max);
                reply_markdown(bot, msg, format!("🎲 Random number: {}", num)).await?;
            }
        }

        Command::Password(len) => {
            if !(2..=128).contains(&len) {
                reply_markdown(bot, msg, "❌ Length must be 2–128".to_string()).await?;
            } else {
                let pw = {
                    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=".chars().collect();
                    let mut rng = rand::thread_rng();
                    (0..len)
                        .map(|_| chars[rng.gen_range(0..chars.len())])
                        .collect::<String>()
                };

                reply_markdown(bot, msg, format!("🔑 Password ({} chars):\n{}", len, pw)).await?;
            }
        }

        Command::Bc(expr) => {
            if expr.trim().is_empty() {
                reply_markdown(bot, msg, "❌ Usage: /bc 2+2*3 or /bc sqrt(16)".to_string()).await?;
            } else {
                match run_bc(&expr).await {
                    Ok(result) => {
                        reply_markdown(bot, msg, format!("📊 bc result:\n{}", result.trim()))
                            .await?;
                    }
                    Err(e) => {
                        reply_markdown(bot, msg, format!("❌ bc error: {}", e)).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn echo_text_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    // 1. Sticker echo
    if let Some(sticker) = msg.sticker() {
        bot.send_sticker(msg.chat.id, InputFile::file_id(sticker.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 2. Photo echo
    if let Some(photos) = msg.photo() {
        if let Some(largest) = photos.last() {
            bot.send_photo(msg.chat.id, InputFile::file_id(largest.file.id.clone()))
                .reply_to(msg.id)
                .await
                .map(|_| ())?;
            return Ok(());
        }
    }

    // 3. Text echo
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            return Ok(()); // command already handled
        }
        reply_markdown(bot, msg.clone(), format!("📢 : {text}")).await?;
    }

    Ok(())
}

async fn reply_markdown(bot: Bot, msg: Message, text: String) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, escape(&text))
        .parse_mode(ParseMode::MarkdownV2)
        .reply_to(msg.id)
        .await
        .map(|_| ())
}

async fn run_bc(expr: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = TokioProcessCommand::new("bc")
        .arg("-l")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(expr.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        // stdin is dropped here → EOF is sent to bc
    }

    let output = child.wait_with_output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() && stderr.is_empty() {
        Ok(stdout)
    } else {
        // Now catches both non-zero exit AND error messages on stderr (the common case)
        Err(format!(
            "bc error (exit {:?}): {}\nstdout was: {}",
            output.status.code(),
            if stderr.is_empty() {
                "(no stderr)".to_string()
            } else {
                stderr
            },
            stdout.trim()
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
        assert!(
            res.is_err(),
            "bc should return Err on invalid input (syntax error reported via stderr)"
        );
    }
}
