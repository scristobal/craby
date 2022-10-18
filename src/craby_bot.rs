use std::sync::Arc;

use dotenv::dotenv;
use tokio::sync::mpsc::{Receiver, Sender};

use log::debug;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::jobs_channels::{JobRequest, JobResult};

pub struct CrabyBot {
    bot: teloxide::Bot,
    tx: Sender<JobRequest>,
    rx: Receiver<JobResult>,
}

impl CrabyBot {
    pub fn build_from_env(rx: Receiver<JobResult>, tx: Sender<JobRequest>) -> Self {
        match dotenv() {
            Ok(_) => debug!("Loaded .env file"),
            Err(_) => debug!("No .env file found. Falling back to environment variables"),
        }

        log::info!("Starting bot...");

        let bot = teloxide::Bot::from_env();

        Self { bot, rx, tx }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let bot = Arc::new(self.bot);

        let forward_result = || {
            let mut rx = self.rx;
            let bot = Arc::clone(&bot);

            tokio::spawn(async move {
                loop {
                    let result = rx.recv();

                    match result.await {
                        Some(result) => {
                            if let Some(error) = result.error {
                                log::error!("Received an error from results channel: {}", error);
                                if let Err(error) = bot
                                    .send_message(result.channel_id, format!("Error: {}", error))
                                    .await
                                {
                                    log::error!("Error sending error message: {}", error);
                                }
                            } else if let Some(url) = result.url {
                                if let Err(error) = bot.send_message(result.channel_id, url).await {
                                    log::error!("Error sending error message: {}", error);
                                }
                            }
                        }
                        None => {
                            todo!()
                        }
                    }
                }
            })
        };

        forward_result();

        teloxide::commands_repl(
            Arc::clone(&bot),
            move |bot: Bot, msg: Message, cmd: Command| {
                let tx = self.tx.clone();
                async move {
                    answer(bot, msg, cmd, tx).await?;
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
    bot: teloxide::Bot,
    msg: Message,
    cmd: Command,
    tx: Sender<JobRequest>,
) -> ResponseResult<()> {
    match cmd {
        Command::Make(prompt) => {
            log::info!(
                "User {} requested {}",
                msg.chat.username().unwrap_or("unknown"),
                prompt
            );

            let job = JobRequest {
                prompt,
                channel_id: msg.chat.id.to_string(),
            };

            if let Err(error) = tx.send(job).await {
                log::error!("Failed to send job to worker channel: {}", error);
                bot.send_message(msg.chat.id, format!("Error: {}", error))
                    .await?;
            }
        }
    };

    Ok(())
}
