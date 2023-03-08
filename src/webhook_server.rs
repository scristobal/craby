use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use warp::{hyper::body::Bytes, Filter};

pub struct WebhookServer {
    tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>,
}

impl WebhookServer {
    pub fn new(tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>) -> Self {
        WebhookServer { tx_results }
    }

    pub async fn run(self, addr: impl Into<SocketAddr>) {
        let use_tx_results = warp::any().map(move || Arc::clone(&self.tx_results));

        let process_entry =
            |id: String,
             body: Bytes,
             tx_map: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>| {
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
            .and(use_tx_results)
            .map(process_entry);

        let health = warp::get()
            .and(warp::path!("health-check"))
            .map(warp::reply);

        let app = warp::any().and(webhooks.or(health));

        warp::serve(app).run(addr).await;
    }
}
