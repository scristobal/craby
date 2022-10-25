#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use log;

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use tokio::sync::{Mutex, Notify};
use warp::Filter;

use crate::models::replicate_api;

const MODEL_VERSION: &str = "a9758cbfbd5f3c2094457d996681af52552901775aa2d6dd0b17fd15df959bef";

const MODEL_URL: &str = "https://api.replicate.com/v1/predictions";

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub prompt: String,
    pub seed: Option<u32>,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
}

#[derive(Serialize, Debug)]
struct PredictionRequest {
    version: String,
    input: Input,
    webhook_completed: Option<String>,
}

pub struct Connector {
    client: Client,
    notifiers: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
    predictions: Arc<Mutex<HashMap<String, replicate_api::Response>>>,
}

impl Connector {
    pub fn new() -> Self {
        let client = Client::new();

        let predictions = Arc::new(Mutex::new(HashMap::new()));
        let notifiers = Arc::new(Mutex::new(HashMap::new()));

        let predictions_server = Arc::clone(&predictions);
        let notifiers_server = Arc::clone(&notifiers);

        tokio::spawn(async { start_server(predictions_server, notifiers_server).await });

        Connector {
            client,
            notifiers,
            predictions,
        }
    }

    pub async fn request(
        &self,
        input: Input,
        id: &String,
    ) -> Result<replicate_api::Response, String> {
        self.model_request(&input, &id)
            .await
            .map_err(|e| format!("job:{} status:error server error {}", id, e))?;

        let notifier = Arc::new(Notify::new());

        {
            let notifiers = &mut self.notifiers.lock().await;
            notifiers.insert(id.clone(), Arc::clone(&notifier));
        }

        notifier.notified().await;
        log::debug!("job:{} status:notified", id);

        let predictions = &mut self.predictions.lock().await;

        predictions.remove(id).map(|p| p.clone()).ok_or(format!(
            "job:{} status:error unable to find prediction result",
            &id
        ))
    }

    async fn model_request(
        &self,
        input: &Input,
        id: &String,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let webhook = std::env::var("WEBHOOK_URL")
            .expect("env variable WEBHOOK_URL should be set to public address");

        let body = PredictionRequest {
            version: MODEL_VERSION.to_string(),
            input: input.clone(),
            webhook_completed: Some(format!("{}webhook/{}", webhook, id)),
        };

        let body = serde_json::to_string(&body).unwrap();

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

pub async fn start_server(
    predictions: Arc<Mutex<HashMap<String, replicate_api::Response>>>,
    notifiers: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
) {
    let use_predictions = warp::any().map(move || Arc::clone(&predictions));
    let use_notifiers = warp::any().map(move || Arc::clone(&notifiers));

    let process_entry =
        |id: String,
         body: replicate_api::Response,
         predictions: Arc<Mutex<HashMap<String, replicate_api::Response>>>,
         notifiers: Arc<Mutex<HashMap<String, Arc<Notify>>>>| {
            log::debug!("job:{} status:processed from webhook", id);

            tokio::spawn(async move {
                let predictions = &mut predictions.lock().await;
                predictions.insert(id.clone(), body);

                let notifiers = &mut notifiers.lock().await;
                let notifier = notifiers.remove(&id);

                match notifier {
                    Some(notifier) => notifier.notify_one(),
                    None => log::error!("job:{} status:error there is no notifier registered", id),
                }
            });

            warp::http::StatusCode::OK
        };

    let webhooks = warp::post()
        .and(warp::path!("webhook" / String))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(use_predictions)
        .and(use_notifiers)
        .map(process_entry);

    let health = warp::get()
        .and(warp::path!("health-check"))
        .map(warp::reply);

    let app = warp::any().and(webhooks.or(health));

    warp::serve(app).run(([0, 0, 0, 0], 8080)).await;
}
