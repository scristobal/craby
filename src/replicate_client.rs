use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client, Url,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use tracing::info;
use uuid::Uuid;
use warp::hyper::body::Bytes;

use crate::errors::{AnswerError, ConnectorError};

const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

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

#[derive(Debug)]
pub struct ReplicateClient {
    client: Client,
    token: String,
    webhook_url: Url,
    tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
}

impl ReplicateClient {
    pub fn new(
        webhook_url: Url,
        token: String,
        tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
    ) -> Self {
        let client = Client::new();

        ReplicateClient {
            client,
            token,
            tx_results,
            webhook_url,
        }
    }

    pub async fn stable_diffusion(&self, prompt: String) -> Result<Url, AnswerError> {
        let input = stable_diffusion::Input {
            prompt,
            num_inference_steps: None,
            seed: None,
            guidance_scale: None,
        };

        let id = Uuid::new_v4();

        let mut webhook_completed = self.webhook_url.clone();

        webhook_completed
            .path_segments_mut()
            .map_err(|_| AnswerError::ParsingURL)?
            .extend(&["webhook", &id.to_string()]);

        info!("{}", webhook_completed.as_str());

        type R = Request<stable_diffusion::Input>;

        let request = R {
            version: stable_diffusion::MODEL_VERSION.to_string(),
            input,
            webhook_completed: Some(webhook_completed.as_str().to_string()),
        };

        let response: Response<stable_diffusion::Input, stable_diffusion::Output> =
            self.request(request, id.to_string()).await?;

        if let Some(error) = response.error {
            return Err(AnswerError::ConnectorError(ConnectorError::ReplicateApi(
                error,
            )));
        }

        let img = response
            .output
            .ok_or(AnswerError::ShouldNotBeNull("output was null".to_string()))?;

        let img = img.last().ok_or(AnswerError::ShouldNotBeNull(
            "output image array was empty".to_string(),
        ))?;

        let url = Url::parse(img)?;

        Ok(url)
    }

    pub async fn dalle_mini(&self, prompt: String) -> Result<Url, AnswerError> {
        let input = dalle_mini::Input {
            text: prompt,
            seed: None,
            grid_size: Some(3),
        };

        let id = Uuid::new_v4();

        let mut webhook_completed = self.webhook_url.clone();

        webhook_completed
            .path_segments_mut()
            .map_err(|_| AnswerError::ParsingURL)?
            .extend(&["webhook", &id.to_string()]);

        info!("{}", webhook_completed.as_str());

        type R = Request<dalle_mini::Input>;

        let request = R {
            version: dalle_mini::MODEL_VERSION.to_string(),
            input,
            webhook_completed: Some(webhook_completed.as_str().to_string()),
        };

        let response: Response<dalle_mini::Input, dalle_mini::Output> =
            self.request(request, id.to_string()).await?;

        if let Some(error) = response.error {
            return Err(AnswerError::ConnectorError(ConnectorError::ReplicateApi(
                error,
            )));
        }

        let img = response
            .output
            .ok_or(AnswerError::ShouldNotBeNull("output was null".to_string()))?;

        let img = img.last().ok_or(AnswerError::ShouldNotBeNull(
            "output image array was empty".to_string(),
        ))?;

        let url = url::Url::parse(img)?;

        Ok(url)
    }

    async fn request<Request: serde::Serialize, Response: for<'a> serde::Deserialize<'a>>(
        &self,
        request: Request,
        id: String,
    ) -> Result<Response, ConnectorError> {
        self.api_call(&request).await?;

        let (tx, rx) = oneshot::channel::<Bytes>();

        {
            let tx_map = &mut self.tx_results.lock().await;
            tx_map.insert(id.clone(), tx);
        }

        let res = rx.await?;

        let res = serde_json::from_slice::<Response>(&res).unwrap();

        Ok(res)
    }

    async fn api_call<Request: serde::Serialize>(
        &self,
        request: &Request,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(MODEL_URL.to_string())
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, "Token ".to_string() + &self.token)
            .json(request)
            .send()
            .await
    }
}
