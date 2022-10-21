use std::sync::Arc;

use dotenv::dotenv;

use log::debug;
use teloxide::{prelude::*, utils::command::BotCommands, RequestError};

use crate::connector::{Connector, Input};

pub fn build_from_env() -> teloxide::Bot {
    match dotenv() {
        Ok(_) => debug!("Loaded .env file"),
        Err(_) => debug!("No .env file found. Falling back to environment variables"),
    }

    log::info!("Starting bot...");

    teloxide::Bot::from_env()
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

pub async fn run(bot: teloxide::Bot, connector: Arc<Connector>) -> Result<(), RequestError> {
    teloxide::commands_repl(
        bot,
        move |bot: Bot, msg: Message, cmd: Command| {
            let connector = Arc::clone(&connector);

            async move {
                answer(bot, msg, cmd, connector).await;
                Ok(())
            }
        },
        Command::ty(),
    )
    .await;

    Ok(())
}

async fn answer(bot: teloxide::Bot, msg: Message, cmd: Command, connector: Arc<Connector>) {
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

            match connector.request(input, msg.chat.id.to_string()).await {
                Ok(response) => {
                    match bot
                        .send_message(msg.chat.id.to_string(), format!("{:?}", response))
                        .await
                    {
                        Ok(_) => log::info!("Job {} completed", msg.chat.id.to_string()),
                        Err(e) => log::error!("Error on delivery {}", e),
                    };
                }
                Err(e) => {
                    log::error!("Error on request {}", e)
                }
            }
        }
    }
}
