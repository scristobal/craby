use craby::craby_r8::CrabyBot;
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(async move {
        let bot = CrabyBot::new_from_env();
        bot.run().await
    });

    tokio::spawn(async move {
        let sever = craby::webhooks::new_server().expect("Failed to start webhook server");
        sever.await
    });

    tokio::signal::ctrl_c().await
}
