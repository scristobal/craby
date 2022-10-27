use crate::connector;
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
    log::info!("new job from {}", msg.chat.username().unwrap_or("unknown"));

    match cmd {
        Command::StableD(prompt) => {
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
        }
        Command::DalleM(prompt) => {
            let response = connector.new_dalle_mini(prompt).await;

            match response {
                Ok(response) => match response.error {
                    None => {
                        let img = response.output;

                        match img {
                            Some(img) => {
                                let img = img.last();
                                match img {
                                    Some(img) => match url::Url::parse(&img) {
                                        Ok(img) => {
                                            bot.send_photo(
                                                msg.chat.id.to_string(),
                                                InputFile::url(img),
                                            )
                                            .caption(response.input.text.to_string())
                                            .await?;
                                        }
                                        Err(e) => {
                                            log::error!("error invalid output url: {}", e)
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            _ => {}
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
            }
        }
    };

    Ok(())
}
