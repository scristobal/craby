use craby::{bot, connector::Connector};
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

    let bot = bot::build_from_env();

    tokio::spawn(async { bot::run(bot, connector).await });

    tokio::signal::ctrl_c().await
}
