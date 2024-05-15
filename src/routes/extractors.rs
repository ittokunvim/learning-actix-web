use actix_web::{get, post, web, error, Result, Responder, HttpRequest, HttpResponse};
use serde::Deserialize;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::Cell;

#[derive(Deserialize)]
pub struct Extractors {
    pub id: u32,
    pub username: String,
}

#[derive(Deserialize)]
pub struct PostInfo {
    pub post_id: u32,
    pub friend: String,
}

#[derive(Deserialize)]
struct QueryStruct {
    name: String,
}

#[derive(Deserialize)]
struct JsonStruct {
    name: String,
}

#[derive(Deserialize)]
struct FormData {
    username: String,
}

#[derive(Clone)]
pub struct StateStruct {
    pub local_count: Cell<usize>,
    pub global_count: Arc<AtomicUsize>,
}

#[get("/extractors")]
async fn extractors(path: web::Path<(String, String)>, info: web::Json<Extractors>) -> impl Responder {
    let path = path.into_inner();
    format!("{} {} {} {}", path.0, path.1, info.id, info.username)
}

#[get("/posts/{post_id}/{friend}")]
async fn post_friend(req: HttpRequest) -> Result<String> {
    let name: String = req.match_info().get("friend").unwrap().parse().unwrap();
    let postid: i32 = req.match_info().query("post_id").parse().unwrap();

    Ok(format!("Welcome {}, post_id: {}", name, postid))
}

#[get("/query")]
async fn query(info: web::Query<QueryStruct>) -> String {
    format!("Welcome {}", info.name)
}

#[post("/json")]
async fn json(info: web::Json<JsonStruct>) -> Result<String> {
    Ok(format!("Welcome {}", info.name))
}

#[post("/form")]
async fn form(form: web::Form<FormData>) -> Result<String> {
    Ok(format!("Welcome {}", form.username))
}

#[get("/count")]
async fn show_count(data: web::Data<StateStruct>) -> impl Responder {
    format!("count: {}", data.local_count.get())
}

#[get("/add-one")]
async fn add_one(data: web::Data<StateStruct>) -> impl Responder {
    data.global_count.fetch_add(1, Ordering::Relaxed);

    let count = data.local_count.get();
    data.local_count.set(count + 1);

    format!("Count: {}", data.local_count.get())
}

fn json_config() -> web::JsonConfig {
    web::JsonConfig::default()
        .limit(4096)
        .error_handler(|err, _req| {
            error::InternalError::from_response(
                err,
                HttpResponse::Conflict().finish()
            )
            .into()
        })
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    let state_counter = web::Data::new(StateStruct {
        local_count: Cell::new(0),
        global_count: Arc::new(AtomicUsize::new(0)),
    });

    config.app_data(json_config);
    config.app_data(state_counter);
    config.service(extractors);
    config.service(post_friend);
    config.service(query);
    config.service(json);
    config.service(form);
    config.service(show_count);
    config.service(add_one);
}
