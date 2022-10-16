use std::sync::Arc;

use dotenv::dotenv;

use crate::{r8client::R8Client, requests};
use log::debug;
use teloxide::{prelude::*, utils::command::BotCommands};

pub struct CrabyBot {
    bot: teloxide::Bot,
    state: Arc<requests::Requests>,
    client: Arc<R8Client>,
}

impl CrabyBot {
    pub fn new_from_env(state: Arc<requests::Requests>) -> Self {
        match dotenv() {
            Ok(_) => debug!("Loaded .env file"),
            Err(_) => debug!("No .env file found. Falling back to environment variables"),
        }

        log::info!("Starting bot...");

        let bot = teloxide::Bot::from_env();

        let client = Arc::new(R8Client::new());

        Self { bot, state, client }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        teloxide::commands_repl(
            self.bot,
            move |bot: Bot, msg: Message, cmd: Command| {
                let client = Arc::clone(&self.client);
                let state = Arc::clone(&self.state);

                async move {
                    answer(bot, msg, cmd, &client, &state).await?;
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

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    client: &R8Client,
    state: &requests::Requests,
) -> ResponseResult<()> {
    match cmd {
        Command::Make(prompt) => {
            log::info!(
                "User {} requested {}",
                msg.chat.username().unwrap_or("unknown"),
                prompt
            );

            state.increment().await;

            let count = state.read().await;

            client.request(prompt.clone()).await;

            bot.send_message(msg.chat.id, format!("Request {} is a {}", count, prompt))
                .await?
        }
    };

    Ok(())
}
