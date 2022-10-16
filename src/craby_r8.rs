use std::sync::Arc;

use dotenv::dotenv;

use crate::r8client::R8Client;
use log::debug;
use teloxide::{prelude::*, utils::command::BotCommands};

pub struct CrabyBot {
    bot: teloxide::Bot,
}

impl CrabyBot {
    pub fn new_from_env() -> Self {
        match dotenv() {
            Ok(_) => debug!("Loaded .env file"),
            Err(_) => debug!("No .env file found. Falling back to environment variables"),
        }

        log::info!("Starting bot...");

        let bot = teloxide::Bot::from_env();
        Self { bot }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let client = Arc::new(R8Client::new());

        teloxide::commands_repl(
            self.bot,
            move |bot: Bot, msg: Message, cmd: Command| {
                let client = Arc::clone(&client);
                async move {
                    answer(bot, msg, cmd, &client).await?;
                    Ok(())
                }
            },
            Command::ty(),
        )
        .await;

        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]

enum Command {
    #[command(description = "Create an image using Stable Diffusion v1.4")]
    Make(String),
}

async fn answer(bot: Bot, msg: Message, cmd: Command, client: &R8Client) -> ResponseResult<()> {
    match cmd {
        Command::Make(prompt) => {
            log::info!(
                "User {} requested {}",
                msg.chat.username().unwrap_or("unknown"),
                prompt
            );

            client.request(prompt.clone()).await;

            bot.send_message(msg.chat.id, format!("Requested a {}", prompt))
                .await?
        }
    };

    Ok(())
}
