use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Error, Debug)]
pub enum AnswerError {
    #[error("bot unable to request")]
    BotRequest(#[from] teloxide::RequestError),
    #[error("could not parse a url")]
    ParseError(#[from] url::ParseError),
    #[error("a field should not be empty, but it was")]
    ShouldNotBeNull(String),
    #[error("there was a problem processing the request")]
    ConnectorError(#[from] ConnectorError),
}

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("replicate api responded with error")]
    ApiError(String),
    #[error("http client error")]
    ClientError(#[from] reqwest::Error),
    #[error("internal channel error")]
    ChannelError(#[from] RecvError),
    #[error("the response did not match the request")]
    ResponseDidNotMatchError,
}
