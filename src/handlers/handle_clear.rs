use crate::utils::clear_conversation::clear_user_conversation;
use crate::utils::reply_markdown::reply_markdown;
use crate::utils::upsert_user::upsert_user_and_get_id;
use sqlx::SqlitePool;
use teloxide::{Bot, prelude::*, types::Message};

pub async fn handle_clear(bot: Bot, msg: Message, pool: SqlitePool) -> ResponseResult<()> {
    let user = match &msg.from {
        Some(u) => u,
        None => {
            reply_markdown(bot, msg, "❌ Cannot identify user.".to_string(), &pool).await?;
            return Ok(());
        }
    };

    let internal_id = match upsert_user_and_get_id(&pool, user).await {
        Ok(id) => id,
        Err(e) => {
            reply_markdown(bot, msg, format!("❌ Failed to get user ID: {}", e), &pool).await?;
            return Ok(());
        }
    };

    match clear_user_conversation(&pool, internal_id).await {
        Ok(count) => {
            let msg_text = if count == 0 {
                "✅ Your conversation was already empty.".to_string()
            } else {
                format!("✅ Cleared {} message(s) from your history.", count)
            };
            reply_markdown(bot, msg, msg_text, &pool).await?;
        }
        Err(e) => {
            reply_markdown(
                bot,
                msg,
                format!("❌ Failed to clear conversation: {}", e),
                &pool,
            )
            .await?;
        }
    }

    Ok(())
}
