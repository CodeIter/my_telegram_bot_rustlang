use crate::commands::Command;
use crate::handlers::{
    handle_gemini::handle_gemini, handle_glm5::handle_glm5, handle_textqr::handle_textqr,
    handle_ytdl::handle_ytdl, handle_ytdlmp3::handle_ytdlmp3,
};
use crate::utils::reply_markdown::reply_markdown;
use crate::utils::save_message::save_message;
use crate::utils::upsert_user::upsert_user_and_get_id;
use base64::{Engine as _, engine::general_purpose};
use percent_encoding::percent_decode_str;
use rand::Rng;
use sqlx::SqlitePool;
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, prelude::*, types::Message};

use crate::handlers::handle_clear::handle_clear;

pub async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    pool: SqlitePool,
) -> ResponseResult<()> {
    // ── Save incoming user command/message ──
    if let Some(user) = &msg.from
        && let Ok(internal_id) = upsert_user_and_get_id(&pool, user).await
    {
        let content = msg.text().map_or_else(|| "".to_string(), |t| t.to_string());
        let _ = save_message(&pool, internal_id, content, false, "text").await;
    }

    match cmd {
        Command::Help => {
            reply_markdown(bot, msg, Command::descriptions().to_string(), &pool).await?;
        }

        Command::Start => {
            reply_markdown(
                bot,
                msg,
                "👋 Hello! I'm your Rust 🦀 AI assistant bot.\nHow can i help you today?\nUse /help for commands list".to_string(),
                &pool,
            )
            .await?;
        }

        Command::Clear => {
            handle_clear(bot, msg, pool).await?;
        }

        Command::Echo(text) => {
            reply_markdown(bot, msg, format!("📢 : {text}"), &pool).await?;
        }

        Command::UrlDecode(encoded) => {
            let decoded = percent_decode_str(&encoded).decode_utf8_lossy().to_string();
            reply_markdown(bot, msg, format!("🔓 Decoded URL:\n{}", decoded), &pool).await?;
        }

        Command::TextBase64Encode(text) => {
            let encoded = general_purpose::STANDARD.encode(text.as_bytes());
            reply_markdown(bot, msg, format!("🔼 Base64 encoded:\n{}", encoded), &pool).await?;
        }

        Command::TextBase64Decode(encoded) => match general_purpose::STANDARD.decode(&encoded) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => {
                    reply_markdown(bot, msg, format!("🔽 Base64 decoded:\n{}", decoded), &pool)
                        .await?;
                }
                Err(_) => {
                    reply_markdown(bot, msg, "❌ Not valid UTF-8".to_string(), &pool).await?;
                }
            },
            Err(_) => {
                reply_markdown(bot, msg, "❌ Invalid Base64".to_string(), &pool).await?;
            }
        },

        Command::Rng(min, max) => {
            if min == 0 || max == 0 || min > max {
                reply_markdown(
                    bot,
                    msg,
                    "❌ Use: /rng 1 100 (min > 0, max > min)".to_string(),
                    &pool,
                )
                .await?;
            } else {
                let num = rand::thread_rng().gen_range(min..=max);
                reply_markdown(bot, msg, format!("🎲 Random number: {}", num), &pool).await?;
            }
        }

        Command::Password(len) => {
            if !(2..=128).contains(&len) {
                reply_markdown(bot, msg, "❌ Length must be 2–128".to_string(), &pool).await?;
            } else {
                let pw = {
                    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=".chars().collect();
                    let mut rng = rand::thread_rng();
                    (0..len)
                        .map(|_| chars[rng.gen_range(0..chars.len())])
                        .collect::<String>()
                };
                reply_markdown(
                    bot,
                    msg,
                    format!("🔑 Password ({} chars):\n{}", len, pw),
                    &pool,
                )
                .await?;
            }
        }

        Command::Bc(expr) => {
            if expr.trim().is_empty() {
                reply_markdown(
                    bot,
                    msg,
                    "❌ Usage: /bc 2+2*3 or /bc sqrt(16)".to_string(),
                    &pool,
                )
                .await?;
            } else {
                match crate::utils::run_bc::run_bc(&expr).await {
                    Ok(result) => {
                        reply_markdown(
                            bot,
                            msg,
                            format!("📊 bc result:\n{}", result.trim()),
                            &pool,
                        )
                        .await?;
                    }
                    Err(e) => {
                        reply_markdown(bot, msg, format!("❌ bc error: {}", e), &pool).await?;
                    }
                }
            }
        }

        Command::Ytdl(url) => {
            handle_ytdl(bot, msg, url, pool).await?;
        }

        Command::YtdlMp3(url) => {
            handle_ytdlmp3(bot, msg, url, pool).await?;
        }

        Command::TextQr(text) => {
            handle_textqr(bot, msg, text, pool).await?;
        }

        Command::Gemini3(prompt) => {
            handle_gemini(bot, msg, prompt, "gemini-3-flash-preview", pool).await?;
        }

        Command::Gemini2(prompt) => {
            handle_gemini(bot, msg, prompt, "gemini-2.5-flash", pool).await?;
        }

        Command::Glm5Ai(prompt) => {
            handle_glm5(bot, msg, prompt, false, pool).await?;
        }

        Command::Glm5AiReasoning(prompt) => {
            handle_glm5(bot, msg, prompt, true, pool).await?;
        }
    }
    Ok(())
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
}
