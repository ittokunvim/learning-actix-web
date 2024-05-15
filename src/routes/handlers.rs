use actix_web::{get, web, http, body, Result, Error, Either, Responder, HttpRequest, HttpResponse};
use serde::Serialize;
use futures::{future::ok, stream::once};

#[derive(Serialize)]
struct CustomType {
    name: &'static str,
}

impl Responder for CustomType {
    type Body = body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
        .content_type(http::header::ContentType::json())
            .body(body)
    }
}

type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>;

#[get("/responder")]
async fn responder(_req: HttpRequest) -> String {
    "Hello World!".to_owned()
}

#[get("/responder2")]
async fn responder_2(_req: HttpRequest) -> impl Responder {
    web::Bytes::from_static(b"Hello World!")
}

#[get("custom-type")]
async fn custom_type() -> impl Responder {
    CustomType { name: "ittokun" }
}

#[get("/stream")]
async fn stream() -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(body)
}

#[get("either")]
async fn either() -> RegisterResult {
    if true {
        Either::Left(HttpResponse::BadRequest().body("Bad data"))
    } else {
        Either::Right(Ok("Hello!"))
    }
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(responder);
    config.service(responder_2);
    config.service(custom_type);
    config.service(stream);
    config.service(either);
}
