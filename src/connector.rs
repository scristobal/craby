#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use log::debug;

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client, Error, Response,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use tokio::sync::Mutex;
use warp::Filter;

const MODEL_VERSION: &str = "a9758cbfbd5f3c2094457d996681af52552901775aa2d6dd0b17fd15df959bef";

const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

#[derive(Deserialize, Debug)]
pub struct PredictionResponse {
    completed_at: Option<String>,
    created_at: Option<String>,
    error: Option<String>,
    hardware: String,
    id: String,
    input: Input,
    logs: String,
    metrics: Metrics,
    output: Option<Vec<String>>,
    started_at: Option<String>,
    status: String,
    urls: Urls,
    version: String,
    webhook_completed: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Metrics {
    predict_time: f32,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub prompt: String,
    pub seed: Option<u32>,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
}

#[derive(Deserialize, Debug)]
struct Urls {
    get: String,
    cancel: String,
}

#[derive(Serialize, Debug)]
struct PredictionRequest {
    version: String,
    input: Input,
    webhook_completed: Option<String>,
}

pub struct Connector {
    predictions: Arc<Mutex<HashMap<String, PredictionResponse>>>,
    client: Arc<Client>,
}

impl Connector {
    pub fn new() -> Self {
        let predictions = Arc::new(Mutex::new(HashMap::new()));
        let client = Arc::new(Client::new());

        Connector {
            predictions,
            client,
        }
    }

    pub async fn run(self) {
        let predictions_filter = warp::any().map(move || Arc::clone(&self.predictions));

        let webhooks = warp::post()
            .and(warp::path::param())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and(predictions_filter.clone())
            .map(
                |id: String,
                 body: PredictionResponse,
                 predictions: Arc<Mutex<HashMap<String, PredictionResponse>>>| {
                    debug!("Got a webhook from {} with body {:?}", id, body);

                    tokio::spawn(async move {
                        let predictions = &mut predictions.lock().await;
                        predictions.insert(id, body);
                    });

                    ""
                },
            );

        warp::serve(webhooks).run(([127, 0, 0, 1], 8080)).await;
    }

    pub async fn request(&self, input: Input, id: String) -> Result<PredictionResponse, Error> {
        todo!();
    }

    async fn model_request(&self, input: Input, id: String) -> Result<Response, Error> {
        let webhook = std::env::var("WEBHOOK_URL")
            .expect("WEBHOOK_URL must be set and point to current address");

        let client = Arc::clone(&self.client);

        let body = PredictionRequest {
            version: MODEL_VERSION.to_string(),
            input: input.clone(),
            webhook_completed: Some(format!("{}/webhook/{}", webhook, id)),
        };

        let body = serde_json::to_string(&body).unwrap();

        let token =
            std::env::var("R8_TOKEN").expect("Replicate's token must be set at R8_TOKEN var");

        let response = client
            .post(MODEL_URL.to_string())
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, "Token ".to_string() + &token)
            .body(body)
            .send()
            .await;

        debug!("{:#?}", response);

        response
    }
}
