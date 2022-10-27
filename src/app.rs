use std::sync::Arc;

use teloxide::{types::Message, utils::command::BotCommands, Bot, RequestError};

use crate::{
    bot::{answer_cmd_repl, Command},
    connector,
};

pub async fn run(bot: teloxide::Bot, connector: connector::Connector) -> Result<(), RequestError> {
    let connector = Arc::new(connector);

    teloxide::commands_repl(
        bot,
        move |bot: Bot, msg: Message, cmd: Command| {
            let connector = Arc::clone(&connector);

            async move {
                answer_cmd_repl(bot, msg, cmd, connector).await?;
                Ok(())
            }
        },
        Command::ty(),
    )
    .await;

    Ok(())
}
