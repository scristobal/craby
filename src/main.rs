use std::sync::Arc;

use dotenv::dotenv;

use log::debug;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};

use tokio::main as async_main;

use unescape::unescape;

#[async_main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenv() {
        Ok(_) => debug!("Loaded .env file"),
        Err(_) => debug!("No .env file found. Falling back to environment variables"),
    }

    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    teloxide::commands_repl(bot, answer, Command::ty()).await;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct JSONResponse {
    status: String,
    output: Vec<String>,
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

            let params = format!("{{\"input\": {{ \"prompt\" : \"{}\" }} }}", prompt);

            debug!("{:?}", unescape(&params));

            let response = client
                .post(url)
                .header(CONTENT_TYPE, "application/json")
                .body(unescape(&params).unwrap())
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
