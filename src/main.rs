use craby::{app, bot, connector::Connector};
use dotenv::dotenv;
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(_) => log::info!("No .env file found. Falling back to environment variables"),
    }

    let connector = Connector::new();

    log::info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(async { app::run(bot, connector).await });

    tokio::signal::ctrl_c().await
}
