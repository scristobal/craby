use dotenv::dotenv;
use teloxide::prelude::*;
use tokio::main as async_main;

#[async_main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, msg.text().unwrap_or("what!?"))
            .await?;
        log::info!(
            "Echoed message from {} in {}: {}",
            msg.chat.id,
            msg.chat.username().unwrap_or("unknown"),
            msg.text().unwrap_or("what!?")
        );
        Ok(())
    })
    .await;
}
