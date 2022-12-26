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

    let (result, prompt) = match cmd {
        Command::StableD(prompt) => (connector.stable_diffusion(prompt.clone()).await, prompt),
        Command::DalleM(prompt) => (connector.dalle_mini(prompt.clone()).await, prompt),
    };

    match result {
        Err(e) => match e {
            AnswerError::BotRequest(e) => Err(e),
            AnswerError::ParseError(e) => Ok(log::error!("error parsing an url: {}", e)),
            AnswerError::ShouldNotBeNull(e) => Ok(log::error!("field should not be null: {}", e)),
            AnswerError::ConnectorError(e) => Ok(log::error!("connector error: {}", e)),
        },
        Ok(url) => {
            bot.send_photo(msg.chat.id.to_string(), InputFile::url(url))
                .caption(prompt)
                .await?;
            Ok(())
        }
    }
}
