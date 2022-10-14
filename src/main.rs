use craby::craby_r8::CrabyBot;
use tokio::main as async_main;

#[async_main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = CrabyBot::new_from_env();

    bot.run().await
}
