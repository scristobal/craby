use std::{collections::HashMap, sync::Arc};

use log;

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};

use tokio::sync::{Mutex, Notify};
use warp::Filter;

use crate::{models, replicate};

pub struct Connector {
    client: Client,
    notifiers: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
    predictions: Arc<Mutex<HashMap<String, models::Response>>>,
}

impl Connector {
    pub fn new() -> Self {
        let client = Client::new();

        let predictions = Arc::new(Mutex::new(HashMap::new()));
        let notifiers = Arc::new(Mutex::new(HashMap::new()));

        let predictions_server = Arc::clone(&predictions);
        let notifiers_server = Arc::clone(&notifiers);

        tokio::spawn(async { start_webhook_server(predictions_server, notifiers_server).await });

        Connector {
            client,
            notifiers,
            predictions,
        }
    }

    pub async fn request(
        &self,
        request: models::Request,
        id: &String,
    ) -> Result<models::Response, String> {
        self.model_request(&request)
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
        request: &models::Request,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let body = serde_json::to_string(&request).unwrap();

        let token = std::env::var("R8_TOKEN")
            .expect("en variable R8_TOKEN should be set to a valid replicate.com token");

        self.client
            .post(replicate::MODEL_URL.to_string())
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, "Token ".to_string() + &token)
            .body(body)
            .send()
            .await
    }
}

pub async fn start_webhook_server(
    predictions: Arc<Mutex<HashMap<String, models::Response>>>,
    notifiers: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
) {
    let use_predictions = warp::any().map(move || Arc::clone(&predictions));
    let use_notifiers = warp::any().map(move || Arc::clone(&notifiers));

    let process_entry =
        |id: String,
         body: models::Response,
         predictions: Arc<Mutex<HashMap<String, models::Response>>>,
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
