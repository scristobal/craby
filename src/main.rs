use craby::{craby_bot::CrabyBot, r8_connector::Connector};

use std::io::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let (tx_jobs, rx_jobs) = mpsc::channel(1);

    let (tx_results, rx_results) = mpsc::channel(1);

    let bot = CrabyBot::build_from_env(rx_results, tx_jobs);

    tokio::spawn(async move { bot.run().await });

    let connector = Connector::new();
    tokio::spawn(async move { connector.run().await });

    tokio::signal::ctrl_c().await
}
