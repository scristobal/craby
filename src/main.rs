use craby::craby_r8::CrabyBot;
use std::io::Result;
use tokio::main as async_main;

#[async_main]
async fn main() -> Result<()> {
    let bot = CrabyBot::new_from_env();

    tokio::spawn(async move { bot.run().await });

    let sever = craby::webhooks::new_server().expect("Failed to start webhook server");

    tokio::spawn(async move { sever.await });

    Ok(())
}
