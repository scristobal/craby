use std::sync::Arc;

use dotenv::dotenv;

use log::debug;
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};

use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};

const R8S_VERSION: &str = "a9758cbfbd5f3c2094457d996681af52552901775aa2d6dd0b17fd15df959bef";

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

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let client = Arc::new(R8SClient::new());

        teloxide::commands_repl(
            self.bot,
            move |bot: Bot, msg: Message, cmd: Command| {
                let client = Arc::clone(&client);
                async move {
                    client.answer(bot, msg, cmd).await?;
                    Ok(())
                }
            },
            Command::ty(),
        )
        .await;

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
    version: String,
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

struct R8SClient {
    client: Client,
}

impl R8SClient {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }

    pub async fn answer(&self, bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
        match cmd {
            Command::Make(prompt) => {
                let url = std::env::var("COG_URL").expect("COG_URL must be set");

                log::info!(
                    "User {} requested {}",
                    msg.chat.username().unwrap_or("unknown"),
                    prompt
                );

                let input = InputParams { prompt: &prompt };

                let body = JSONRequest {
                    version: R8S_VERSION.to_string(),
                    input,
                };

                let body = serde_json::to_string(&body).unwrap();

                let token = std::env::var("COG_TOKEN").expect("COG_TOKEN must be set");

                let response = self
                    .client
                    .post(url)
                    .header(CONTENT_TYPE, "application/json")
                    .header(AUTHORIZATION, "Token ".to_string() + &token)
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
}
