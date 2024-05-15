use actix_web::{get, guard, http, web::{self, service}, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct PathInfo {
    id: u32,
    username: String,
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello")
}

#[get("/show")]
async fn show_users() -> HttpResponse {
    HttpResponse::Ok().body("Show users")
}

#[get("/show/{id}")]
async fn user_detail(path: web::Path<(u32,)>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User detail: {}", path.into_inner().0))
}

#[get("/match/{v1}/{v2}")]
async fn match_info(req: HttpRequest) -> HttpResponse {
    let v1: u8 = req.match_info().get("v1").unwrap().parse().unwrap();
    let v2: u8 = req.match_info().query("v2").parse().unwrap();
    let (v3, v4): (u8, u8) = req.match_info().load().unwrap();
    HttpResponse::Ok().body(format!("Values {} {} {} {}", v1, v2, v3, v4))
}

#[get("/path/{username}/{id}")]
async fn path_info(info: web::Path<(String, u32)>) -> HttpResponse {
    let info = info.into_inner();
    HttpResponse::Ok().body(format!("Welcome {}! id: {}", info.0, info.1))
}

#[get("/v2/path/{username}/{id}")]
async fn path_info_v2(info: web::Path<PathInfo>) -> HttpResponse {
    HttpResponse::Ok().body(format!("Welcome {}! id: {}", info.username, info.id))
}

#[get("/generate-resource-url")]
async fn generate_resource_urls(req: HttpRequest) -> HttpResponse {
    let url = req.url_for("foo", ["1", "2", "3"]).unwrap();

    HttpResponse::Found()
        .insert_header((http::header::LOCATION, url.as_str()))
        .finish()
}

#[get("/external-resources")]
async fn external_resources(req: HttpRequest) -> HttpResponse {
    let url = req.url_for("youtube", ["oHg5SJYRHA0"]).unwrap();
    assert_eq!(url.as_str(), "https://youtube.com/watch/oHg5SJYRHA0");

    HttpResponse::Ok().body(url.to_string())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    // Resource configuration
    cfg.route("/url-dispatch", web::get().to(index));
    cfg.route("/url-dispatch/user", web::post().to(index));
    cfg.service(web::resource("/url-dispatch/prefix").to(index));
    cfg.service(
        web::resource("url-dispatch/user/{name}")
            .name("user_detail")
            .guard(guard::Header("content-type", "application/json"))
            .route(web::get().to(HttpResponse::Ok))
            .route(web::put().to(HttpResponse::Ok)),
    );
    // Configuring a Route
    cfg.service(
        web::resource("/url-dispatch/path").route(
            web::route()
                .guard(guard::Get())
                .guard(guard::Header("content-type", "text/plain"))
                .to(HttpResponse::Ok),
        ),
    );
    cfg.service(
        // Scoping Routes
        web::scope("url-dispatch")
            .service(user_detail)
            // Match information
            .service(match_info)
            // Path information extractor
            .service(path_info)
            .service(path_info_v2)
            // Generating resource URLs
            .service(
                web::resource("/generate-resource-urls/{a}/{b}/{c}")
                    .name("foo")
                    .guard(guard::Get())
                    .to(index),
            )
            .service(generate_resource_urls)
            // External resources
            .service(external_resources)
            // Path normalization
            .route("/path-normalize", web::get().to(index)),
    );
    cfg.external_resource("youtube", "https://youtube.com/watch/{video_id}");
}
