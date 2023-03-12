use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use tracing::info;
use warp::{hyper::body::Bytes, Filter};

type LookupTable = Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>;

pub struct WebhookServer {
    tx_results: LookupTable,
}

impl WebhookServer {
    pub fn new(tx_results: Arc<Mutex<HashMap<String, oneshot::Sender<Bytes>>>>) -> Self {
        WebhookServer { tx_results }
    }

    pub async fn run(self, addr: impl Into<SocketAddr>) {
        let use_tx_results = warp::any().map(move || Arc::clone(&self.tx_results));

        let process_entry = |id: String, body: Bytes, tx_map: LookupTable| async move {
            info!("webhook received: {}", id);

            let tx_map = &mut tx_map.lock().await;
            let tx = tx_map.remove(&id);

            let res = tx.and_then(|tx| tx.send(body).ok());

            match res {
                Some(_) => Ok(warp::http::StatusCode::OK),
                None => Err(warp::reject::not_found()),
            }
        };

        let webhooks = warp::post()
            .and(warp::path!("webhook" / String))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::bytes())
            .and(use_tx_results)
            .and_then(process_entry);

        let health = warp::get()
            .and(warp::path!("health-check"))
            .map(warp::reply);

        let app = warp::any().and(webhooks.or(health));

        warp::serve(app).run(addr).await;
    }
}
