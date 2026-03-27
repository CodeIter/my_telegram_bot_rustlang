use crate::utils::reply_markdown::reply_markdown;
use image::Luma;
use qrcode::QrCode;
use rand::Rng;
use std::path::PathBuf;
use teloxide::{
    Bot,
    prelude::*,
    sugar::request::RequestReplyExt,
    types::{InputFile, Message},
};
use tokio::fs;

pub async fn handle_textqr(bot: Bot, msg: Message, text: String) -> ResponseResult<()> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        reply_markdown(
            bot,
            msg,
            "❌ Usage: /textqr your text here (text is required)".to_string(),
        )
        .await?;
        return Ok(());
    }

    let id: u64 = rand::thread_rng().r#gen::<u64>();
    let filepath = format!("textqr_{}.png", id);
    let path = PathBuf::from(filepath);

    match QrCode::new(trimmed.as_bytes()) {
        Ok(code) => {
            let img = code
                .render::<Luma<u8>>()
                .module_dimensions(20, 20)
                .quiet_zone(true)
                .build();

            if let Err(e) = img.save(&path) {
                reply_markdown(bot, msg, format!("❌ Failed to save QR image: {}", e)).await?;
                return Ok(());
            }

            let caption = format!(
                "✅ QR Code generated with Rust 🦀\n🔤 {}",
                if trimmed.len() > 200 {
                    format!("{}...", &trimmed[..200])
                } else {
                    trimmed.to_string()
                }
            );

            if let Err(e) = bot
                .send_photo(msg.chat.id, InputFile::file(path.clone()))
                .caption(caption)
                .reply_to(msg.id)
                .await
            {
                let _ = fs::remove_file(&path).await;
                reply_markdown(bot, msg, format!("⚠️ Upload failed (file cleaned): {}", e)).await?;
                return Ok(());
            }

            let _ = fs::remove_file(&path).await; // success cleanup
        }
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Failed to generate QR: {} (text may be too long)", e),
            )
            .await?;
        }
    }
    Ok(())
}
