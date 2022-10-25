use crate::models::replicate_api;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub const MODEL_VERSION: &str = "a9758cbfbd5f3c2094457d996681af52552901775aa2d6dd0b17fd15df959bef";
pub const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub prompt: String,
    pub seed: Option<u32>,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
}

pub type Output = Option<Vec<String>>;

pub type StableDiffusionRequest = replicate_api::PredictionRequest<Input>;

pub type StableDiffusionResponse = replicate_api::PredictionResponse<Input, Output>;

impl StableDiffusionResponse {
    pub fn caption(&self) -> String {
        self.input.prompt.to_string()
    }

    pub fn imgs(&self) -> Option<Vec<String>> {
        self.output.clone()
    }
}
