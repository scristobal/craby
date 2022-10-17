use log::debug;
use warp::Filter;

use crate::r8client::PredictionResponse;

pub async fn new_server() {
    let webhooks = warp::post()
        .and(warp::path::param())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|id: String, body: PredictionResponse| {
            debug!("Got a webhook from {} with body {:?}", id, body);
            ""
        });

    warp::serve(webhooks).run(([127, 0, 0, 1], 8080)).await
}

/*
use actix_web::{dev::Server, post, web, App, HttpResponse, HttpServer, Responder};
use log::debug;
use serde::Deserialize;

#[derive(Deserialize)]
struct Id {
    id: String,
}

#[post("/webhook/{id}")]
async fn webhook(path: web::Path<Id>, req_body: String) -> impl Responder {
    debug!("Got a webhook from {} with body {}", path.id, req_body);

    HttpResponse::Ok()
}

pub fn new_server() -> Result<Server, std::io::Error> {
    Ok(HttpServer::new(|| App::new().service(webhook))
        .bind(("127.0.0.1", 8080))?
        .run())
}
*/
