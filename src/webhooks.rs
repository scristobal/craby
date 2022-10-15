use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use log::debug;
use serde::Deserialize;
use std::io::Result;

#[derive(Deserialize)]
struct Id {
    id: String,
}

#[post("/webhook/{id}")]
async fn webhook(path: web::Path<Id>, req_body: String) -> impl Responder {
    debug!("Got a webhook from {} with body {}", path.id, req_body);

    HttpResponse::Ok()
}

pub struct WebhookServer {}

impl WebhookServer {
    pub async fn start() -> Result<()> {
        HttpServer::new(|| App::new().service(webhook))
            .bind(("127.0.0.1", 8080))?
            .run()
            .await?;
        Ok(())
    }
}
