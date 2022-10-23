use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("bot unable to request")]
    BotRequest(#[from] teloxide::RequestError),
}
