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
