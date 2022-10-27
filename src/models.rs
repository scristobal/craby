use serde::{Deserialize, Serialize};

mod base;
mod dalle_mini;
mod stable_diffusion;

#[derive(Serialize)]
#[serde(untagged)]
pub enum Request {
    StableDiffusion(stable_diffusion::Request),
    DalleMini(dalle_mini::Request),
}

pub fn new_stable_diffusion(id: &String, prompt: String) -> Request {
    let webhook = std::env::var("WEBHOOK_URL")
        .expect("env variable WEBHOOK_URL should be set to public address");

    let input = stable_diffusion::Input::new(prompt);

    Request::StableDiffusion(stable_diffusion::Request {
        version: stable_diffusion::MODEL_VERSION.to_string(),
        input,
        webhook_completed: Some(format!("{}webhook/{}", webhook, id)),
    })
}

pub fn new_dalle_mini(id: &String, prompt: String) -> Request {
    let webhook = std::env::var("WEBHOOK_URL")
        .expect("env variable WEBHOOK_URL should be set to public address");

    let input = dalle_mini::Input::new(prompt);

    Request::DalleMini(dalle_mini::Request {
        version: dalle_mini::MODEL_VERSION.to_string(),
        input,
        webhook_completed: Some(format!("{}webhook/{}", webhook, id)),
    })
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Response {
    StableDiffusion(stable_diffusion::Response),
    DalleMini(dalle_mini::Response),
}

impl Response {
    pub fn error(&self) -> Option<String> {
        match self {
            Response::StableDiffusion(response) => response.error(),
            Response::DalleMini(response) => response.error(),
        }
    }
    pub fn caption(&self) -> String {
        match self {
            Response::StableDiffusion(response) => response.caption(),
            Response::DalleMini(response) => response.caption(),
        }
    }

    pub fn imgs(&self) -> Option<Vec<String>> {
        match self {
            Response::StableDiffusion(response) => response.imgs(),
            Response::DalleMini(response) => response.imgs(),
        }
    }
}
