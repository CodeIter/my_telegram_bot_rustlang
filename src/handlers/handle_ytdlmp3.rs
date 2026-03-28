use crate::utils::{reply_markdown::reply_markdown, run_yt_dlp_mp3::run_yt_dlp_mp3};
use rand::Rng;
use std::path::PathBuf;
use teloxide::{
    Bot,
    prelude::*,
    sugar::request::RequestReplyExt,
    types::{InputFile, Message},
};
use tokio::fs;

use crate::utils::{save_message::save_message, upsert_user::upsert_user_and_get_id};
use sqlx::SqlitePool;

pub async fn handle_ytdlmp3(
    bot: Bot,
    msg: Message,
    url: String,
    pool: SqlitePool,
) -> ResponseResult<()> {
    if url.trim().is_empty() || !url.starts_with("http") {
        reply_markdown(
            bot,
            msg,
            "❌ Usage: /ytdlmp3 https://youtu.be/xxx or full YouTube link".to_string(),
            &pool,
        )
        .await?;
        return Ok(());
    }

    let id: u64 = rand::thread_rng().r#gen::<u64>();
    let output_template = format!("ytdlmp3_{}.%(ext)s", id);

    match run_yt_dlp_mp3(&url, &output_template).await {
        Ok(_) => {
            let filepath = format!("ytdlmp3_{}.mp3", id);
            let path = PathBuf::from(filepath);

            if path.exists() {
                if let Err(e) = bot
                    .send_audio(msg.chat.id, InputFile::file(path.clone()))
                    .caption(format!("✅ MP3 Downloaded with yt-dlp 🦀\n🔗 {}", url))
                    .reply_to(msg.id)
                    .await
                {
                    let _ = fs::remove_file(&path).await;
                    reply_markdown(
                        bot,
                        msg,
                        format!("⚠️ Upload timed out (file cleaned): {}", e),
                        &pool,
                    )
                    .await?;
                    return Ok(());
                } else {
                    // success → save bot media
                    if let Some(user) = &msg.from {
                        if let Ok(internal_id) = upsert_user_and_get_id(&pool, user).await {
                            let _ = save_message(
                                &pool,
                                internal_id,
                                "<sent> file audio".to_string(),
                                true,
                                "audio",
                            )
                            .await;
                        }
                    }
                }

                let _ = fs::remove_file(&path).await;
            } else {
                reply_markdown(
                    bot,
                    msg,
                    "❌ Downloaded but file not found (maybe no audio)".to_string(),
                    &pool,
                )
                .await?;
            }
        }
        Err(e) => {
            reply_markdown(bot, msg, format!("❌ yt-dlp failed: {}", e), &pool).await?;
        }
    }
    Ok(())
}
