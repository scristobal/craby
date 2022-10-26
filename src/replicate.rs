use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

#[derive(Deserialize, Debug, Clone)]
pub struct Response<I, O> {
    completed_at: Option<String>,
    created_at: Option<String>,
    pub error: Option<String>,
    hardware: String,
    id: String,
    pub input: I,
    logs: String,
    metrics: Metrics,
    pub output: O,
    started_at: Option<String>,
    status: String,
    urls: Urls,
    version: String,
    webhook_completed: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct Metrics {
    predict_time: f32,
}

#[derive(Deserialize, Debug, Clone)]
struct Urls {
    get: String,
    cancel: String,
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
pub struct Request<I> {
    pub version: String,
    pub input: I,
    pub webhook_completed: Option<String>,
}
