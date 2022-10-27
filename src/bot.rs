use crate::{connector, errors::Error};
use log;
use std::sync::Arc;

use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands, RequestError};

pub fn build_from_env() -> teloxide::Bot {
    log::info!("Starting bot...");

    teloxide::Bot::from_env()
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Create an image from text using Stable Diffusion v1.4")]
    StableD(String),
    #[command(description = "Create an image from text using Dalle Mini")]
    DalleM(String),
}

pub async fn answer_cmd_repl(
    bot: teloxide::Bot,
    msg: Message,
    cmd: Command,
    connector: Arc<connector::Connector>,
) -> Result<(), RequestError> {
    log::info!("new job from {}", msg.chat.username().unwrap_or("unknown"));

    let result = match cmd {
        Command::StableD(prompt) => answer_stable_diffusion(&connector, prompt, &bot, &msg).await,
        Command::DalleM(prompt) => answer_dalle_mini(&connector, prompt, &bot, &msg).await,
    };

    match result {
        Err(e) => match e {
            Error::BotRequest(e) => Err(e),
            Error::ParseError(e) => Ok(log::error!("error parsing an url: {}", e)),
            Error::ShouldNotBeNull(e) => Ok(log::error!("field should not be null: {}", e)),
        },
        _ => Ok(()),
    }
}

async fn answer_dalle_mini(
    connector: &Arc<connector::Connector>,
    prompt: String,
    bot: &Bot,
    msg: &Message,
) -> Result<(), Error> {
    let response = connector.new_dalle_mini(prompt).await;
    match response {
        Ok(response) => match response.error {
            None => {
                let img = response
                    .output
                    .ok_or(Error::ShouldNotBeNull("output was null".to_string()))?;

                let img = img.last().ok_or(Error::ShouldNotBeNull(
                    "output image array was empty".to_string(),
                ))?;

                let url = url::Url::parse(&img)?;

                bot.send_photo(msg.chat.id.to_string(), InputFile::url(url))
                    .caption(response.input.text.to_string())
                    .await?;
            }
            Some(e) => {
                log::error!("remote api error: {}", e);
                bot.send_message(msg.chat.id.to_string(), e).await?;
            }
        },
        Err(e) => {
            log::error!("connector error: {}", e)
        }
    };
    Ok(())
}

async fn answer_stable_diffusion(
    connector: &Arc<connector::Connector>,
    prompt: String,
    bot: &Bot,
    msg: &Message,
) -> Result<(), Error> {
    let response = connector.new_stable_diffusion(prompt).await;
    match response {
        Ok(response) => match response.error {
            None => {
                let imgs: &Vec<String> = &response.output.into_iter().flatten().collect();

                for img in imgs {
                    match url::Url::parse(&img) {
                        Ok(img) => {
                            bot.send_photo(msg.chat.id.to_string(), InputFile::url(img))
                                .caption(response.input.prompt.to_string())
                                .await?;
                        }
                        Err(e) => {
                            log::error!("error invalid output url: {}", e)
                        }
                    }
                }
            }
            Some(e) => {
                log::error!("remote api error: {}", e);
                bot.send_message(msg.chat.id.to_string(), e).await?;
            }
        },
        Err(e) => {
            log::error!("connector error: {}", e)
        }
    };
    Ok(())
}
