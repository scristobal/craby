use std::sync::Arc;

use dotenv::dotenv;

use log;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands, RequestError};

use crate::{connector, models};

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
    #[command(description = "Create an image from text using Stable Diffusion v1.4")]
    StableD(String),
    #[command(description = "Create an image from text using Dalle Mini")]
    DalleM(String),
}

pub async fn run(bot: teloxide::Bot, connector: connector::Connector) -> Result<(), RequestError> {
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
    connector: Arc<connector::Connector>,
) -> Result<(), RequestError> {
    let id = msg.chat.id.to_string();

    log::info!("job:{} status:init ", &id,);

    let request = match cmd {
        Command::StableD(prompt) => models::new_stable_diffusion(&id, prompt),
        Command::DalleM(prompt) => models::new_dalle_mini(&id, prompt),
    };

    match connector.request(request, &id).await {
        Ok(response) => match response.error() {
            None => {
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
            Some(e) => {
                log::error!("job:{} status:error on response {}", &id, e);
                bot.send_message(id.to_string(), e).await?;
            }
        },
        Err(e) => {
            log::error!("job:{} status:error on request {}", &id, e)
        }
    }
    Ok(())
}
