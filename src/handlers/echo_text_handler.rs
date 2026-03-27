use crate::utils::reply_markdown::reply_markdown;
use teloxide::{
    Bot,
    prelude::*,
    sugar::request::RequestReplyExt,
    types::{InputFile, Message},
};

pub async fn echo_text_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    // 1. Sticker echo
    if let Some(sticker) = msg.sticker() {
        bot.send_sticker(msg.chat.id, InputFile::file_id(sticker.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 2. GIF / Animation echo
    if let Some(animation) = msg.animation() {
        bot.send_animation(msg.chat.id, InputFile::file_id(animation.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 3. Video echo
    if let Some(video) = msg.video() {
        bot.send_video(msg.chat.id, InputFile::file_id(video.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 4. Photo echo
    if let Some(photos) = msg.photo()
        && let Some(largest) = photos.last()
    {
        bot.send_photo(msg.chat.id, InputFile::file_id(largest.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 5. Voice message echo
    if let Some(voice) = msg.voice() {
        bot.send_voice(msg.chat.id, InputFile::file_id(voice.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 6. File / Document echo
    if let Some(document) = msg.document() {
        bot.send_document(msg.chat.id, InputFile::file_id(document.file.id.clone()))
            .reply_to(msg.id)
            .await
            .map(|_| ())?;
        return Ok(());
    }

    // 7. Text echo
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            return Ok(()); // command already handled
        }
        reply_markdown(bot, msg.clone(), format!("📢 : {text}")).await?;
    }

    Ok(())
}
