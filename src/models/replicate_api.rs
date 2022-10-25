use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::stable_diffusion::{StableDiffusionRequest, StableDiffusionResponse};

#[derive(Deserialize, Debug, Clone)]
pub struct PredictionResponse<I, O> {
    completed_at: Option<String>,
    created_at: Option<String>,
    error: Option<String>,
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
pub struct PredictionRequest<I> {
    version: String,
    input: I,
    webhook_completed: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Request {
    StableDiffusion(StableDiffusionRequest),
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Response {
    StableDiffusion(StableDiffusionResponse),
}

impl Response {
    pub fn caption(&self) -> String {
        match self {
            Response::StableDiffusion(response) => response.caption(),
        }
    }

    pub fn imgs(&self) -> Option<Vec<String>> {
        match self {
            Response::StableDiffusion(response) => response.output.clone(),
        }
    }
}
