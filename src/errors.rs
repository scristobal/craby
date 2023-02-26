use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Error, Debug)]
pub enum AnswerError {
    #[error("bot unable to request")]
    BotRequest(#[from] teloxide::RequestError),
    #[error("could not parse a url")]
    UrlParse(#[from] url::ParseError),
    #[error("a field should not be empty, but it was")]
    ShouldNotBeNull(String),
    #[error("there was a problem processing the request")]
    ConnectorError(#[from] ConnectorError),
    #[error("no request")]
    NoRequest,
}

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("replicate api responded with error")]
    ReplicateApi(String),
    #[error("http client error")]
    HttpClient(#[from] reqwest::Error),
    #[error("internal channel error")]
    InternalChannel(#[from] RecvError),
    #[error("the response did not match the request")]
    ResponseDidNotMatch,
}
