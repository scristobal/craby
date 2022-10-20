use std::sync::Arc;

use dotenv::dotenv;

use log::debug;
use reqwest::Error;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::connector::{Connector, Input, PredictionResponse};

pub struct CrabyBot {
    bot: teloxide::Bot,
    connector: Arc<Connector>,
}

impl CrabyBot {
    pub fn build_from_env(connector: Arc<Connector>) -> Self {
        match dotenv() {
            Ok(_) => debug!("Loaded .env file"),
            Err(_) => debug!("No .env file found. Falling back to environment variables"),
        }

        log::info!("Starting bot...");

        let bot = teloxide::Bot::from_env();

        Self { bot, connector }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let bot = Arc::new(self.bot);

        teloxide::commands_repl(
            Arc::clone(&bot),
            move |bot: Bot, msg: Message, cmd: Command| {
                let connector = Arc::clone(&self.connector);

                async move {
                    answer(bot, msg, cmd, connector).await?;
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
    _bot: teloxide::Bot,
    msg: Message,
    cmd: Command,
    connector: Arc<Connector>,
) -> Result<PredictionResponse, Error> {
    match cmd {
        Command::Make(prompt) => {
            log::info!(
                "User {} requested {}",
                msg.chat.username().unwrap_or("unknown"),
                prompt
            );

            let input = Input {
                prompt,
                num_inference_steps: None,
                seed: None,
                guidance_scale: None,
            };

            connector.request(input, msg.chat.id.to_string()).await
        }
    }
}
