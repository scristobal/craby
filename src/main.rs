use craby::{connector::Connector, craby_bot::CrabyBot};

use std::{io::Result, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let connector = Arc::new(Connector::new());

    tokio::spawn(async { connector.run().await });

    let bot = CrabyBot::build_from_env(connector);

    tokio::spawn(async { bot.run().await });

    tokio::signal::ctrl_c().await
}
