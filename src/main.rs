use craby::{app, replicate_client::ReplicateClient, webhook_server::WebhookServer};
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

    let public_url = std::env::var("PUBLIC_URL")
        .expect("env variable PUBLIC_URL should be set to public address");

    let public_url = url::Url::parse(&public_url).expect("PUBLIC_URL should be a valid url");

    let token = std::env::var("R8_TOKEN")
        .expect("en variable R8_TOKEN should be set to a valid replicate.com token");

    let tx_results = Arc::new(Mutex::new(HashMap::<String, oneshot::Sender<Bytes>>::new()));

    log::info!("Setting up webhook server...");
    let webhook_server = WebhookServer::new(tx_results.clone());

    log::info!("Setting up API server...");
    let replicate_client = ReplicateClient::new(public_url, token, tx_results.clone());

    log::info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(async { app::run(bot, replicate_client, webhook_server).await });

    tokio::signal::ctrl_c().await
}
