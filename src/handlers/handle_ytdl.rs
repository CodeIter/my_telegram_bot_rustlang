use crate::utils::{reply_markdown::reply_markdown, run_yt_dlp::run_yt_dlp};
use rand::Rng;
use std::path::PathBuf;
use teloxide::{
    Bot,
    prelude::*,
    sugar::request::RequestReplyExt,
    types::{InputFile, Message},
};
use tokio::fs;

pub async fn handle_ytdl(bot: Bot, msg: Message, url: String) -> ResponseResult<()> {
    if url.trim().is_empty() || !url.starts_with("http") {
        reply_markdown(
            bot,
            msg,
            "❌ Usage: /ytdl https://youtu.be/xxx or full YouTube link".to_string(),
        )
        .await?;
        return Ok(());
    }

    let id: u64 = rand::thread_rng().r#gen::<u64>();
    let output_template = format!("ytdl_{}.%(ext)s", id);

    match run_yt_dlp(&url, &output_template).await {
        Ok(_) => {
            let filepath = format!("ytdl_{}.mp4", id);
            let path = PathBuf::from(filepath);

            if path.exists() {
                if let Err(e) = bot
                    .send_video(msg.chat.id, InputFile::file(path.clone()))
                    .caption(format!("✅ Downloaded with yt-dlp 🦀\n🔗 {}", url))
                    .reply_to(msg.id)
                    .await
                {
                    let _ = fs::remove_file(&path).await;
                    reply_markdown(
                        bot,
                        msg,
                        format!("⚠️ Upload timed out (file cleaned): {}", e),
                    )
                    .await?;
                    return Ok(());
                }

                let _ = fs::remove_file(&path).await; // success path
            } else {
                reply_markdown(
                    bot,
                    msg,
                    "❌ Downloaded but file not found (maybe no video)".to_string(),
                )
                .await?;
            }
        }
        Err(e) => {
            reply_markdown(bot, msg, format!("❌ yt-dlp failed: {}", e)).await?;
        }
    }
    Ok(())
}
