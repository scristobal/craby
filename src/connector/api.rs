use super::{dalle_mini, stable_diffusion};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Request {
    StableDiffusion(stable_diffusion::Request),
    DalleMini(dalle_mini::Request),
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Response {
    StableDiffusion(stable_diffusion::Response),
    DalleMini(dalle_mini::Response),
}
