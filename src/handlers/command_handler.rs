use crate::commands::Command;
use crate::handlers::{
    handle_gemini::handle_gemini, handle_glm5::handle_glm5, handle_textqr::handle_textqr,
    handle_ytdl::handle_ytdl, handle_ytdlmp3::handle_ytdlmp3,
};
use crate::utils::reply_markdown::reply_markdown;
use crate::utils::run_bc::run_bc;
use base64::{Engine as _, engine::general_purpose};
use percent_encoding::percent_decode_str;
use rand::Rng;
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, prelude::*, types::Message};

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
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

        Command::Ytdl(url) => {
            handle_ytdl(bot, msg, url).await?;
        }

        Command::YtdlMp3(url) => {
            handle_ytdlmp3(bot, msg, url).await?;
        }

        Command::TextQr(text) => {
            handle_textqr(bot, msg, text).await?;
        }

        Command::Gemini3(prompt) => {
            handle_gemini(bot, msg, prompt, "gemini-3-flash-preview").await?;
        }

        Command::Gemini2(prompt) => {
            handle_gemini(bot, msg, prompt, "gemini-2.5-flash").await?;
        }

        Command::Glm5Ai(prompt) => {
            handle_glm5(bot, msg, prompt, false).await?;
        }

        Command::Glm5AiReasoning(prompt) => {
            handle_glm5(bot, msg, prompt, true).await?;
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
