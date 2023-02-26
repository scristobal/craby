use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client, Url,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;
use warp::{hyper::body::Bytes, Filter};

use crate::{
    api::{self, dalle_mini, stable_diffusion},
    errors::{AnswerError, ConnectorError},
};

const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

pub struct Connector {
    client: Client,
    webhook_url: String,
    results_channel_map: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
}

impl Connector {
    pub fn new() -> Self {
        let webhook = std::env::var("WEBHOOK_URL")
            .expect("env variable WEBHOOK_URL should be set to public address");

        let client = Client::new();

        let results_channel_map =
            Arc::new(Mutex::new(HashMap::<String, oneshot::Sender<Bytes>>::new()));

        let tx_map_clone = Arc::clone(&results_channel_map);

        tokio::spawn(async { start_webhook_server(tx_map_clone).await });

        Connector {
            client,
            results_channel_map,
            webhook_url: webhook,
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

        type Request = api::Request<stable_diffusion::Input>;
        type Response = api::Response<stable_diffusion::Input, stable_diffusion::Output>;

        let request = Request {
            version: stable_diffusion::MODEL_VERSION.to_string(),
            input,
            webhook_completed: Some(format!("{}webhook/{}", &self.webhook_url, &id)),
        };

        let response: Response = self.request(request, id.to_string()).await?;

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

    pub async fn dalle_mini(&self, prompt: String) -> Result<Url, AnswerError> {
        let input = dalle_mini::Input {
            text: prompt,
            seed: None,
            grid_size: Some(3),
        };

        let id = Uuid::new_v4();

        type Request = api::Request<dalle_mini::Input>;
        type Response = api::Response<dalle_mini::Input, dalle_mini::Output>;

        let request = Request {
            version: dalle_mini::MODEL_VERSION.to_string(),
            input,
            webhook_completed: Some(format!("{}webhook/{}", self.webhook_url, &id)),
        };

        let response: Response = self.request(request, id.to_string()).await?;

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
            let tx_map = &mut self.results_channel_map.lock().await;
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
        let body = serde_json::to_string(&request).unwrap();

        let token = std::env::var("R8_TOKEN")
            .expect("en variable R8_TOKEN should be set to a valid replicate.com token");

        self.client
            .post(MODEL_URL.to_string())
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, "Token ".to_string() + &token)
            .body(body)
            .send()
            .await
    }
}

async fn start_webhook_server(
    results_channel_map: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
) {
    let use_tx_map = warp::any().map(move || Arc::clone(&results_channel_map));

    let process_entry =
        |id: String, body: Bytes, tx_map: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>| {
            tokio::spawn(async move {
                let tx_map = &mut tx_map.lock().await;
                let tx = tx_map.remove(&id);

                tx.and_then(|tx| tx.send(body).ok());
            });

            warp::http::StatusCode::OK
        };

    let webhooks = warp::post()
        .and(warp::path!("webhook" / String))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::bytes())
        .and(use_tx_map)
        .map(process_entry);

    let health = warp::get()
        .and(warp::path!("health-check"))
        .map(warp::reply);

    let app = warp::any().and(webhooks.or(health));

    warp::serve(app).run(([0, 0, 0, 0], 8080)).await;
}
