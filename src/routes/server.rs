use actix_web::{get, http, web, Responder, HttpResponse};

use std::time::Duration;

#[get("/sleep")]
async fn sleep() -> impl Responder {
    tokio::time::sleep(Duration::from_secs(5)).await;
    "response"
}

#[get("/quit")]
async fn quit() -> HttpResponse {
    let mut res = HttpResponse::Ok()
        .force_close()
        .finish();

    res.head_mut().set_connection_type(http::ConnectionType::Close);
    res
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(sleep);
    config.service(quit);
}
