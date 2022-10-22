use std::sync::Arc;

use dotenv::dotenv;

use log;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands, RequestError};

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
            let id = msg.chat.id.to_string();

            log::info!(
                "job:{} status:init by user {} prompt {}",
                &id,
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
                    let imgs: Vec<String> = response.imgs().into_iter().flatten().collect();

                    for img in imgs {
                        match url::Url::parse(&img) {
                            Ok(img) => {
                                match bot
                                    .send_photo(msg.chat.id.to_string(), InputFile::url(img))
                                    .caption(response.caption())
                                    .await
                                {
                                    Ok(_) => log::info!(
                                        "job:{} status:completed",
                                        msg.chat.id.to_string()
                                    ),
                                    Err(e) => {
                                        log::error!("job:{} status:error on delivery {}", id, e,)
                                    }
                                };
                            }
                            Err(e) => {
                                log::error!("job:{} status:error invalid output url {}", &id, e)
                            }
                        }
                    }
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
