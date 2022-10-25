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
    DalleMini(String),
}

pub async fn run(bot: teloxide::Bot, connector: Connector) -> Result<(), RequestError> {
    let connector = Arc::new(connector);

    teloxide::commands_repl(
        bot,
        move |bot: Bot, msg: Message, cmd: Command| {
            let connector = Arc::clone(&connector);

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

async fn answer(
    bot: teloxide::Bot,
    msg: Message,
    cmd: Command,
    connector: Arc<Connector>,
) -> Result<(), RequestError> {
    let id = msg.chat.id.to_string();

    log::info!("job:{} status:init ", &id,);

    match cmd {
        Command::Make(prompt) => {
            let input = Input {
                prompt,
                num_inference_steps: None,
                seed: None,
                guidance_scale: None,
            };

            match connector.request(input, &id).await {
                Ok(response) => {
                    let imgs: Vec<String> = response.imgs().into_iter().flatten().collect();

                    for img in imgs {
                        match url::Url::parse(&img) {
                            Ok(img) => {
                                bot.send_photo(id.to_string(), InputFile::url(img))
                                    .caption(response.caption())
                                    .await?;
                            }
                            Err(e) => {
                                log::error!("job:{} status:error invalid output url {}", &id, e)
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("job:{} status:error on request {}", &id, e)
                }
            }
        }
        Command::DalleMini(_) => {
            bot.send_message(id, "not yet implemented").await?;
        }
    }
    Ok(())
}
