use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Debug, Clone)]
pub struct Response<Input, Output> {
    completed_at: Option<String>,
    created_at: Option<String>,
    pub error: Option<String>,
    hardware: Option<String>,
    id: String,
    pub input: Input,
    logs: String,
    metrics: Metrics,
    pub output: Output,
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

pub mod dalle_mini {
    use serde::{Deserialize, Serialize};
    use serde_with::skip_serializing_none;

    pub const MODEL_VERSION: &str =
        "f178fa7a1ae43a9a9af01b833b9d2ecf97b1bcb0acfd2dc5dd04895e042863f1";

    #[skip_serializing_none]
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Input {
        pub text: String,
        pub seed: Option<u32>,
        pub grid_size: Option<u32>,
    }

    pub type Output = Option<Vec<String>>;
}

pub mod stable_diffusion {
    use serde::{Deserialize, Serialize};
    use serde_with::skip_serializing_none;

    pub const MODEL_VERSION: &str =
        "328bd9692d29d6781034e3acab8cf3fcb122161e6f5afb896a4ca9fd57090577";

    #[skip_serializing_none]
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Input {
        pub prompt: String,
        pub seed: Option<u32>,
        pub num_inference_steps: Option<u32>,
        pub guidance_scale: Option<f32>,
    }

    pub type Output = Option<Vec<String>>;
}
