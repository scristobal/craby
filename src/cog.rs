use std::sync::Arc;

use dotenv::dotenv;

use log::debug;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};

pub struct CrabyBot {
    bot: teloxide::Bot,
}

impl CrabyBot {
    pub fn new_from_env() -> Self {
        match dotenv() {
            Ok(_) => debug!("Loaded .env file"),
            Err(_) => debug!("No .env file found. Falling back to environment variables"),
        }

        pretty_env_logger::init();
        log::info!("Starting bot...");

        let bot = teloxide::Bot::from_env();
        Self { bot }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        teloxide::commands_repl(self.bot.clone(), answer, Command::ty()).await;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct JSONResponse {
    output: Vec<String>,
}

#[derive(Serialize, Debug)]
struct InputParams<'a> {
    prompt: &'a String,
}

#[derive(Serialize, Debug)]
struct JSONRequest<'a> {
    input: InputParams<'a>,
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

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let client = Arc::new(reqwest::Client::new());

    match cmd {
        Command::Make(prompt) => {
            let client = Arc::clone(&client);

            let url = std::env::var("COG_URL").expect("COG_URL must be set");

            log::info!(
                "User {} requested {}",
                msg.chat.username().unwrap_or("unknown"),
                prompt
            );

            let input = InputParams { prompt: &prompt };

            let body = JSONRequest { input };

            let body = serde_json::to_string(&body).unwrap();

            let response = client
                .post(url)
                .header(CONTENT_TYPE, "application/json")
                .body(body)
                .send()
                .await?;

            debug!("{:#?}", response);

            match response.status() {
                reqwest::StatusCode::OK => {
                    let json_response: JSONResponse = response.json().await?;
                    debug!("{:#?}", json_response);

                    let img = json_response.output[0].split(",").collect::<Vec<&str>>()[1];

                    let img = base64::decode(img).unwrap();

                    let img = InputFile::memory(img);

                    bot.send_photo(msg.chat.id, img).caption(prompt).await?;
                }
                _ => {
                    bot.send_message(msg.chat.id, "Something went wrong")
                        .await?;
                }
            }
        }
    };

    Ok(())
}
