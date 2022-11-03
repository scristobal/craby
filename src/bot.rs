use crate::{connector, errors::AnswerError};
use log;
use std::sync::Arc;

use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands, RequestError};

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
            AnswerError::BotRequest(e) => Err(e),
            AnswerError::ParseError(e) => Ok(log::error!("error parsing an url: {}", e)),
            AnswerError::ShouldNotBeNull(e) => Ok(log::error!("field should not be null: {}", e)),
            AnswerError::ConnectorError(e) => Ok(log::error!("connector error: {}", e)),
        },
        Ok(()) => Ok(()),
    }
}

async fn answer_dalle_mini(
    connector: &Arc<connector::Connector>,
    prompt: String,
    bot: &Bot,
    msg: &Message,
) -> Result<(), AnswerError> {
    let response = connector.dalle_mini(prompt).await?;

    let img = response
        .output
        .ok_or(AnswerError::ShouldNotBeNull("output was null".to_string()))?;

    let img = img.last().ok_or(AnswerError::ShouldNotBeNull(
        "output image array was empty".to_string(),
    ))?;

    let url = url::Url::parse(&img)?;

    bot.send_photo(msg.chat.id.to_string(), InputFile::url(url))
        .caption(response.input.text.to_string())
        .await?;

    Ok(())
}

async fn answer_stable_diffusion(
    connector: &Arc<connector::Connector>,
    prompt: String,
    bot: &Bot,
    msg: &Message,
) -> Result<(), AnswerError> {
    let response = connector.stable_diffusion(prompt).await?;

    let imgs: &Vec<String> = &response.output.into_iter().flatten().collect();

    for img in imgs {
        let url = url::Url::parse(img)?;

        bot.send_photo(msg.chat.id.to_string(), InputFile::url(url))
            .caption(response.input.prompt.to_string())
            .await?;
    }

    Ok(())
}
