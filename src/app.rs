use std::sync::Arc;

use teloxide::{types::Message, utils::command::BotCommands, Bot};

use crate::{
    bot::{answer_cmd_repl, Command},
    replicate_client, webhook_server,
};

pub async fn run(
    bot: teloxide::Bot,
    replicate_client: replicate_client::ReplicateClient,
    webhooks_server: webhook_server::WebhookServer,
) {
    tokio::spawn(webhooks_server.run(([0, 0, 0, 0], 8080)));

    let replicate_client = Arc::new(replicate_client);

    teloxide::commands_repl(
        bot,
        move |bot: Bot, msg: Message, cmd: Command| {
            let replicate_client = Arc::clone(&replicate_client);

            async move { answer_cmd_repl(bot, msg, cmd, replicate_client).await }
        },
        Command::ty(),
    )
    .await;
}
