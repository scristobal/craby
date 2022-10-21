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
    let notifiers = Arc::new(Mutex::new(HashMap::new()));

    let notifiers_server = Arc::clone(&notifiers);

    tokio::spawn(async { start_server(predictions, notifiers_server).await });

    let connector = Arc::new(Connector::new(notifiers));

    let bot = bot::build_from_env();

    tokio::spawn(async { bot::run(bot, connector).await });

    tokio::signal::ctrl_c().await
}
