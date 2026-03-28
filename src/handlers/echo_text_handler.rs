use crate::utils::reply_markdown::reply_markdown;
use crate::utils::{save_message::save_message, upsert_user::upsert_user_and_get_id};

use sqlx::SqlitePool;
use teloxide::{
    Bot,
    prelude::*,
    sugar::request::RequestReplyExt,
    types::{InputFile, Message},
};

pub async fn echo_text_handler(bot: Bot, msg: Message, pool: SqlitePool) -> ResponseResult<()> {
    // Helper to save media placeholder
    async fn save_media(pool: &SqlitePool, msg: &Message, is_bot: bool, file_type: &str) {
        if let Some(user) = &msg.from {
            if let Ok(internal_id) = upsert_user_and_get_id(pool, user).await {
                let prefix = if is_bot { "<sent>" } else { "<received>" };
                let content = format!("{} file {}", prefix, file_type);
                let _ = save_message(pool, internal_id, content, is_bot, file_type).await;
            }
        }
    }

    // 1. Sticker
    if let Some(sticker) = msg.sticker() {
        let _ = save_media(&pool, &msg, false, "sticker").await;
        let res = bot
            .send_sticker(msg.chat.id, InputFile::file_id(sticker.file.id.clone()))
            .reply_to(msg.id)
            .await;
        if let Ok(_) = res {
            let _ = save_media(&pool, &msg, true, "sticker").await;
        }
        return Ok(());
    }

    // 2. Animation
    if let Some(animation) = msg.animation() {
        let _ = save_media(&pool, &msg, false, "animation").await;
        let res = bot
            .send_animation(msg.chat.id, InputFile::file_id(animation.file.id.clone()))
            .reply_to(msg.id)
            .await;
        if let Ok(_) = res {
            let _ = save_media(&pool, &msg, true, "animation").await;
        }
        return Ok(());
    }

    // 3. Video
    if let Some(video) = msg.video() {
        let _ = save_media(&pool, &msg, false, "video").await;
        let res = bot
            .send_video(msg.chat.id, InputFile::file_id(video.file.id.clone()))
            .reply_to(msg.id)
            .await;
        if let Ok(_) = res {
            let _ = save_media(&pool, &msg, true, "video").await;
        }
        return Ok(());
    }

    // 4. Photo
    if let Some(photos) = msg.photo()
        && let Some(largest) = photos.last()
    {
        let _ = save_media(&pool, &msg, false, "photo").await;
        let res = bot
            .send_photo(msg.chat.id, InputFile::file_id(largest.file.id.clone()))
            .reply_to(msg.id)
            .await;
        if let Ok(_) = res {
            let _ = save_media(&pool, &msg, true, "photo").await;
        }
        return Ok(());
    }

    // 5. Voice
    if let Some(voice) = msg.voice() {
        let _ = save_media(&pool, &msg, false, "voice").await;
        let res = bot
            .send_voice(msg.chat.id, InputFile::file_id(voice.file.id.clone()))
            .reply_to(msg.id)
            .await;
        if let Ok(_) = res {
            let _ = save_media(&pool, &msg, true, "voice").await;
        }
        return Ok(());
    }

    // 6. Document
    if let Some(document) = msg.document() {
        let _ = save_media(&pool, &msg, false, "document").await;
        let res = bot
            .send_document(msg.chat.id, InputFile::file_id(document.file.id.clone()))
            .reply_to(msg.id)
            .await;
        if let Ok(_) = res {
            let _ = save_media(&pool, &msg, true, "document").await;
        }
        return Ok(());
    }

    // 7. Text (non-command)
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            return Ok(()); // command already handled
        }
        // save received text
        if let Some(user) = &msg.from {
            if let Ok(internal_id) = upsert_user_and_get_id(&pool, user).await {
                let _ = save_message(&pool, internal_id, text.to_string(), false, "text").await;
            }
        }
        reply_markdown(bot, msg.clone(), format!("📢 : {text}"), &pool).await?;
    }

    Ok(())
}
