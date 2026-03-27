use crate::utils::markdown_v2_escape::markdown_v2_escape;
use crate::utils::message_split::{MAX_MESSAGE_LENGTH, split_long_message};
use teloxide::{Bot, prelude::*, sugar::request::RequestReplyExt, types::ParseMode};

pub async fn reply_markdown(bot: Bot, msg: Message, text: String) -> ResponseResult<()> {
    let escaped = markdown_v2_escape(&text);
    if escaped.len() <= MAX_MESSAGE_LENGTH {
        if let Err(e) = bot
            .send_message(msg.chat.id, escaped)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_to(msg.id)
            .await
        {
            log::warn!(
                "MarkdownV2 failed for short message, falling back to plain text: {}",
                e
            );
            let _ = bot.send_message(msg.chat.id, text).reply_to(msg.id).await;
        }
        return Ok(());
    }

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
        }
    }
    Ok(())
}
