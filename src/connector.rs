use std::{collections::HashMap, sync::Arc};

use log;

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};

use tokio::sync::{oneshot, Mutex};
use warp::Filter;

use crate::{models, replicate};

type ResultsChannel = oneshot::Sender<models::Response>;

pub struct Connector {
    client: Client,
    results_channel_map: Arc<Mutex<HashMap<String, ResultsChannel>>>,
}

impl Connector {
    pub fn new() -> Self {
        let client = Client::new();

        let results_channel_map = Arc::new(Mutex::new(HashMap::<String, ResultsChannel>::new()));

        let tx_map_clone = Arc::clone(&results_channel_map);

        tokio::spawn(async { start_webhook_server(tx_map_clone).await });

        Connector {
            client,
            results_channel_map,
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

        let (tx, rx) = oneshot::channel::<models::Response>();

        {
            let tx_map = &mut self.results_channel_map.lock().await;
            tx_map.insert(id.clone(), tx);
        }

        rx.await
            .map_err(|e| format!("job:{} status:error on notification {}", &id, e))
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
    results_channel_map: Arc<Mutex<HashMap<String, ResultsChannel>>>,
) {
    let use_tx_map = warp::any().map(move || Arc::clone(&results_channel_map));

    let process_entry =
        |id: String,
         body: models::Response,
         tx_map: Arc<Mutex<HashMap<String, ResultsChannel>>>| {
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
