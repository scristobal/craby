use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("bot unable to request")]
    BotRequest(#[from] teloxide::RequestError),
    #[error("could not parse a url")]
    ParseError(#[from] url::ParseError),
    #[error("a field should not be empty, but it was")]
    ShouldNotBeNull(String),
}
