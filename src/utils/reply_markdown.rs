use crate::utils::markdown_v2_escape::markdown_v2_escape;
use crate::utils::message_split::{MAX_MESSAGE_LENGTH, split_long_message};
use crate::utils::{save_message::save_message, upsert_user::upsert_user_and_get_id};
use sqlx::SqlitePool;
use teloxide::{Bot, prelude::*, sugar::request::RequestReplyExt, types::ParseMode};

pub async fn reply_markdown(
    bot: Bot,
    msg: Message,
    text: String,
    pool: &SqlitePool,
) -> ResponseResult<()> {
    let escaped = markdown_v2_escape(&text);

    let send_success = if escaped.len() <= MAX_MESSAGE_LENGTH {
        let res = bot
            .send_message(msg.chat.id, escaped)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_to(msg.id)
            .await;

        if let Err(e) = res {
            log::warn!(
                "MarkdownV2 failed for short message, falling back to plain text: {}",
                e
            );
            let _ = bot
                .send_message(msg.chat.id, text.clone())
                .reply_to(msg.id)
                .await;
        }
        true
    } else {
        let mut success = true;
        for chunk in split_long_message(&escaped) {
            if let Err(e) = bot
                .send_message(msg.chat.id, chunk.as_str())
                .parse_mode(ParseMode::MarkdownV2)
                .reply_to(msg.id)
                .await
            {
                log::warn!(
                    "MarkdownV2 failed for long message chunk, falling back to plain text: {}",
                    e
                );
                let _ = bot
                    .send_message(msg.chat.id, chunk.as_str())
                    .reply_to(msg.id)
                    .await;
                success = false;
            }
        }
        success
    };

    if send_success {
        save_bot_text(pool, &msg, &text).await;
    }

    Ok(())
}

async fn save_bot_text(pool: &SqlitePool, msg: &Message, text: &str) {
    if let Some(user) = &msg.from {
        if let Ok(internal_id) = upsert_user_and_get_id(pool, user).await {
            let _ = save_message(pool, internal_id, text.to_string(), true, "text").await;
        }
    }
}
