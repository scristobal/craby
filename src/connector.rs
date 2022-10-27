mod api;
mod base;
mod dalle_mini;
mod stable_diffusion;

use log;
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;
use warp::Filter;

const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

type ResultsChannel = oneshot::Sender<api::Response>;

pub struct Connector {
    client: Client,
    webhook_url: String,
    results_channel_map: Arc<Mutex<HashMap<String, ResultsChannel>>>,
}

impl Connector {
    pub fn new() -> Self {
        let webhook = std::env::var("WEBHOOK_URL")
            .expect("env variable WEBHOOK_URL should be set to public address");

        let client = Client::new();

        let results_channel_map = Arc::new(Mutex::new(HashMap::<String, ResultsChannel>::new()));

        let tx_map_clone = Arc::clone(&results_channel_map);

        tokio::spawn(async { start_webhook_server(tx_map_clone).await });

        Connector {
            client,
            results_channel_map,
            webhook_url: webhook,
        }
    }

    pub async fn new_stable_diffusion(
        &self,
        prompt: String,
    ) -> Result<stable_diffusion::Response, String> {
        let input = stable_diffusion::Input {
            prompt,
            num_inference_steps: None,
            seed: None,
            guidance_scale: None,
        };

        let id = Uuid::new_v4();

        let request = api::Request::StableDiffusion(stable_diffusion::Request {
            version: stable_diffusion::MODEL_VERSION.to_string(),
            input,
            webhook_completed: Some(format!("{}webhook/{}", &self.webhook_url, &id)),
        });

        let response = self.request(request, id.to_string()).await?;

        match response {
            api::Response::StableDiffusion(response) => Ok(response),
            _ => Err("error wrong response from server".to_string()),
        }
    }

    pub async fn new_dalle_mini(&self, prompt: String) -> Result<dalle_mini::Response, String> {
        let input = dalle_mini::Input {
            text: prompt,
            seed: None,
            grid_size: Some(3),
        };

        let id = Uuid::new_v4();

        let request = api::Request::DalleMini(dalle_mini::Request {
            version: dalle_mini::MODEL_VERSION.to_string(),
            input,
            webhook_completed: Some(format!("{}webhook/{}", self.webhook_url, &id)),
        });

        let response = self.request(request, id.to_string()).await?;

        match response {
            api::Response::DalleMini(response) => Ok(response),
            _ => Err("error wrong response from server".to_string()),
        }
    }

    pub async fn request(
        &self,
        request: api::Request,
        id: String,
    ) -> Result<api::Response, String> {
        self.model_request(&request)
            .await
            .map_err(|e| format!("model request error: {}", e))?;

        let (tx, rx) = oneshot::channel::<api::Response>();

        {
            let tx_map = &mut self.results_channel_map.lock().await;
            tx_map.insert(id.clone(), tx);
        }

        rx.await
            .map_err(|e| format!("oneshot channel error: {}", e))
    }

    async fn model_request(
        &self,
        request: &api::Request,
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

pub async fn start_webhook_server(
    results_channel_map: Arc<Mutex<HashMap<String, ResultsChannel>>>,
) {
    let use_tx_map = warp::any().map(move || Arc::clone(&results_channel_map));

    let process_entry =
        |id: String, body: api::Response, tx_map: Arc<Mutex<HashMap<String, ResultsChannel>>>| {
            log::debug!("job:{} status:processed from webhook", id);

            tokio::spawn(async move {
                let tx_map = &mut tx_map.lock().await;
                let tx = tx_map.remove(&id);

                match tx {
                    Some(tx) => match tx.send(body) {
                        Ok(_) => return,
                        Err(_) => {
                            log::error!("job:{} status:error on send result", id);
                        }
                    },
                    None => {
                        log::error!("job:{} status:error there is no notifier registered", id);
                    }
                }
            });

            warp::http::StatusCode::OK
        };

    let webhooks = warp::post()
        .and(warp::path!("webhook" / String))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(use_tx_map)
        .map(process_entry);

    let health = warp::get()
        .and(warp::path!("health-check"))
        .map(warp::reply);

    let app = warp::any().and(webhooks.or(health));

    warp::serve(app).run(([0, 0, 0, 0], 8080)).await;
}
