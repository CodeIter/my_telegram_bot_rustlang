
use teloxide::{prelude::*, utils::command::BotCommands};
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting Telegram bot...");

    let bot = Bot::from_env(); // reads TELOXIDE_TOKEN env var

    // Simple command + echo bot
    Command::repl(bot, answer).await;
}

// Define your commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this help text.")]
    Help,
    #[command(description = "Say hello!")]
    Start,
    #[command(description = "Echo any text you send.")]
    Echo(String),
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match cmd {
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions().to_string())
                .await?;
        }
        Command::Start => {
            bot.send_message(chat_id, "👋 Hello! I'm a Rust bot powered by teloxide.\nSend /help to see commands.")
                .await?;
        }
        Command::Echo(text) => {
            bot.send_message(chat_id, format!("📢 You said: {}", text))
                .await?;
        }
    }

    // Fallback: echo any normal text message (even without /echo)
    if let Some(text) = msg.text() {
        if !text.starts_with('/') {
            bot.send_message(chat_id, format!("Echo: {}", text)).await?;
        }
    }

    Ok(())
}

