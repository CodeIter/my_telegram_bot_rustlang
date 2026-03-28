use dotenvy::dotenv;
use std::sync::Arc;
use teloxide::{
    RequestError,
    dispatching::{UpdateHandler, dialogue::InMemStorage},
    filter_command,
    prelude::*,
    types::Update,
    utils::command::BotCommands,
};

use sqlx::SqlitePool;

mod commands;
mod handlers;
mod init_db;
mod utils;

use crate::commands::Command;
use crate::handlers::command_handler::command_handler;
use crate::handlers::echo_text_handler::echo_text_handler;
use crate::init_db::init_db;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("🚀 Starting Telegram bot...");

    let bot = Bot::from_env();

    let pool = match init_db().await {
        Ok(p) => p,
        Err(e) => {
            log::error!("❌ Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    match bot.set_my_commands(Command::bot_commands()).await {
        Ok(_) => log::info!("✅ Bot commands registered (visible in Telegram / menu)"),
        Err(e) => log::error!("Failed to set bot commands: {}", e),
    }

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<()>::new(), pool])
        .default_handler(|upd: Arc<Update>| async move {
            log::warn!("Unhandled update: {upd:#?}");
        })
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<RequestError> {
    dptree::entry().branch(
        Update::filter_message()
            .branch(filter_command::<Command, _>().endpoint(
                |bot: Bot, msg: Message, cmd: Command, pool: SqlitePool| async move {
                    command_handler(bot, msg, cmd, pool).await
                },
            ))
            .branch(Update::filter_message().endpoint(
                |bot: Bot, msg: Message, pool: SqlitePool| async move {
                    echo_text_handler(bot, msg, pool).await
                },
            )),
    )
}
