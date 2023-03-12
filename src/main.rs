use craby::{
    bot_client::{answer_cmd_repl, Command},
    replicate_client::ReplicateClient,
    webhook_server::WebhookServer,
};
use dotenv::dotenv;
use std::io::Result;
use teloxide::{types::Message, utils::command::BotCommands, Bot};
use tracing::info;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use warp::hyper::body::Bytes;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match dotenv() {
        Ok(_) => info!("Loaded .env file"),
        Err(_) => info!("No .env file found. Falling back to environment variables"),
    }

    let public_url = std::env::var("PUBLIC_URL")
        .expect("env variable PUBLIC_URL should be set to public address");

    let public_url = url::Url::parse(&public_url).expect("PUBLIC_URL should be a valid url");

    let token = std::env::var("R8_TOKEN")
        .expect("en variable R8_TOKEN should be set to a valid replicate.com token");

    let tx_results = Arc::new(Mutex::new(HashMap::<String, oneshot::Sender<Bytes>>::new()));

    info!("Setting up webhook server...");
    let webhook_server = WebhookServer::new(tx_results.clone());

    tokio::spawn(webhook_server.run(([0, 0, 0, 0], 8080)));

    info!("Setting up API server...");
    let replicate_client = ReplicateClient::new(public_url, token, tx_results.clone());

    info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(async move {
        let replicate_client = Arc::new(replicate_client);

        teloxide::commands_repl(
            bot,
            move |bot: Bot, msg: Message, cmd: Command| {
                let replicate_client = Arc::clone(&replicate_client);

                async move { answer_cmd_repl(bot, msg, cmd, replicate_client).await }
            },
            Command::ty(),
        )
        .await;
    });

    tokio::signal::ctrl_c().await
}
