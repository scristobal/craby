use std::{collections::HashMap, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use warp::{hyper::body::Bytes, Filter};

pub struct WebhookServer {
    pub url: String,
    tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
}

impl WebhookServer {
    pub fn new_from_env(tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>) -> Self {
        let url = std::env::var("WEBHOOK_URL")
            .expect("env variable WEBHOOK_URL should be set to public address");

        WebhookServer { url, tx_results }
    }

    pub async fn run(self) {
        start_webhook_server(self.tx_results).await
    }
}

async fn start_webhook_server(
    results_channel_map: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
) {
    let use_tx_map = warp::any().map(move || Arc::clone(&results_channel_map));

    let process_entry =
        |id: String, body: Bytes, tx_map: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>| {
            log::info!("webhook received: {}", id);

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
