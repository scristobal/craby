use craby::{app, requests::Requests, webhooks::WebhookServer};
use dotenv::dotenv;
use std::io::Result;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use warp::hyper::body::Bytes;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(_) => log::info!("No .env file found. Falling back to environment variables"),
    }

    let tx_results = Arc::new(Mutex::new(HashMap::<String, oneshot::Sender<Bytes>>::new()));

    log::info!("Setting up webhook server...");
    let webhook_server = WebhookServer::new_from_env(tx_results.clone());

    log::info!("Setting up API server...");
    let connector = Requests::new(webhook_server.url.to_string(), tx_results);

    log::info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(async { app::run(bot, connector, webhook_server).await });

    tokio::signal::ctrl_c().await
}
