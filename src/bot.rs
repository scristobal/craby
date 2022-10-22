use std::sync::Arc;

use dotenv::dotenv;

use log;
use teloxide::{prelude::*, utils::command::BotCommands, RequestError};

use crate::connector::{Connector, Input};

pub fn build_from_env() -> teloxide::Bot {
    match dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(_) => log::info!("No .env file found. Falling back to environment variables"),
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

pub async fn run(bot: teloxide::Bot, connector: Connector) -> Result<(), RequestError> {
    let connector = Arc::new(connector);

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
                "job:{} status:init user {} prompt {}",
                msg.chat.id.to_string(),
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
                        Ok(_) => log::info!("job:{} status:completed", msg.chat.id.to_string()),
                        Err(e) => log::error!(
                            "job:{} status:error on delivery {}",
                            msg.chat.id.to_string(),
                            e
                        ),
                    };
                }
                Err(e) => {
                    log::error!(
                        "job:{} status:error on request {}",
                        msg.chat.id.to_string(),
                        e
                    )
                }
            }
        }
    }
}
