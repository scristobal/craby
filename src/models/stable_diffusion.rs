use super::base;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub const MODEL_VERSION: &str = "a9758cbfbd5f3c2094457d996681af52552901775aa2d6dd0b17fd15df959bef";

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub prompt: String,
    pub seed: Option<u32>,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
}

impl Input {
    pub fn new(prompt: String) -> Self {
        Input {
            prompt,
            num_inference_steps: None,
            seed: None,
            guidance_scale: None,
        }
    }
}

pub type Output = Option<Vec<String>>;

pub type Request = base::Request<Input>;

pub type Response = base::Response<Input, Output>;

impl Response {
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
    pub fn caption(&self) -> String {
        self.input.prompt.to_string()
    }

    pub fn imgs(&self) -> Option<Vec<String>> {
        self.output.clone()
    }
}
