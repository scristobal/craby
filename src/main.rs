use craby::{
    bot,
    connector::{start_server, Connector},
};
use tokio::sync::Mutex;

use std::{collections::HashMap, io::Result, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let predictions = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(async { start_server(predictions) });

    let connector = Arc::new(Connector::new());

    let bot = bot::build_from_env();

    tokio::spawn(async { bot::run(bot, connector) });

    tokio::signal::ctrl_c().await
}
