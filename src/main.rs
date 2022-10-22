use craby::{bot, connector::Connector};

use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let connector = Connector::new();

    let bot = bot::build_from_env();

    tokio::spawn(async { bot::run(bot, connector).await });

    tokio::signal::ctrl_c().await
}
