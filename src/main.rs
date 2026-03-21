
use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::Update,
    utils::command::BotCommands,
    filter_command,
    RequestError,
};
use dotenvy::dotenv;
use std::sync::Arc;   // ← for default_handler

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("🚀 Starting Telegram bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<()>::new()])
        .default_handler(|upd: Arc<Update>| async move {   // ← FIXED: Arc<Update>
            log::warn!("Unhandled update: {upd:#?}");
        })
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<RequestError> {   // ← FIXED: use RequestError (matches your handlers)
    dptree::entry().branch(
        Update::filter_message()
            .branch(filter_command::<Command, _>().endpoint(command_handler))   // commands first
            .branch(Update::filter_message().endpoint(echo_text_handler)),      // plain echo
    )
}

// ── Commands ────────────────────────────────────────────────────────
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Say hello!")]
    Start,
    #[command(description = "Echo any text (but we also echo without command)")]
    Echo(String),
}

async fn command_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match cmd {
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions().to_string()).await?;
        }
        Command::Start => {
            bot.send_message(
                chat_id,
                "👋 Hello! I'm your Rust 🦀 bot powered by teloxide:!\nJust type anything → I will echo it!\nUse /help for commands list",
            )
            .await?;
        }
        Command::Echo(text) => {
            bot.send_message(chat_id, format!("📢 : {text}")).await?;
        }
    }
    Ok(())
}

// ── Echo ANY plain text (no / needed) ───────────────────────────────
async fn echo_text_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if text.starts_with('/') {
            return Ok(()); // already handled by command branch
        }
        bot.send_message(msg.chat.id, format!("📢 : {text}")).await?;
    }
    Ok(())
}

