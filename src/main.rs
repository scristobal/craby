use craby::craby_r8::CrabyBot;
use craby::requests;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let state = Arc::new(requests::Requests {
        counter: Mutex::new(0),
    });

    let bot = CrabyBot::new_from_env(Arc::clone(&state));
    tokio::spawn(async move { bot.run().await });

    let sever = craby::webhooks::new_server().expect("Failed to start webhook server");
    tokio::spawn(async move { sever.await });

    tokio::signal::ctrl_c().await
}
