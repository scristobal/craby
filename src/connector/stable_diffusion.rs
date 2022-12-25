use super::base;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub const MODEL_VERSION: &str = "f178fa7a1ae43a9a9af01b833b9d2ecf97b1bcb0acfd2dc5dd04895e042863f1";

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub prompt: String,
    pub seed: Option<u32>,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
}

pub type Output = Option<Vec<String>>;

pub type Request = base::Request<Input>;
pub type Response = base::Response<Input, Output>;
