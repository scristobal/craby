use dotenv::dotenv;

use log::debug;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use teloxide::{prelude::*, types::InputFile};
use tokio::main as async_main;

use unescape::unescape;

#[derive(Serialize, Deserialize, Debug)]
struct JSONResponse {
    status: String,
    output: Vec<String>,
}

#[async_main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let client = reqwest::Client::new();
    let bot = Bot::from_env();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let client = client.clone();

        async move {
            let url = std::env::var("COG_URL").expect("COG_URL must be set");

            log::info!(
                "User {} requested {}",
                msg.chat.username().unwrap_or("unknown"),
                msg.text().unwrap_or("what!?")
            );

            let params = format!(
                "{{\"input\": {{ \"prompt\" : \"{}\" }} }}",
                msg.text().unwrap_or("what!?")
            );

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

                    bot.send_photo(msg.chat.id, img)
                        .caption(msg.text().unwrap_or("unknown prompt"))
                        .await?;
                }
                _ => {
                    bot.send_message(msg.chat.id, "Something went wrong")
                        .await?;
                }
            }

            Ok(())
        }
    })
    .await;

    Ok(())
}
