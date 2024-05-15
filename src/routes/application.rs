use actix_web::{get, web, guard, Responder, HttpResponse};
use actix_web::post;

use std::sync::Mutex;

pub struct AppStateWithCounter {
    pub app_name: String,
    pub counter: Mutex<i32>,
}

#[get("/")]
async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let app_name = &data.app_name;
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Hello {app_name}, Request number: {counter}")
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/show")]
async fn show_users() -> impl Responder {
    HttpResponse::Ok().body("Alice, Bob, Chris, Dan, Eve")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn app() -> impl Responder {
    "Hello world!"
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    let counter = web::Data::new(AppStateWithCounter {
        app_name: String::from("Actix Web"),
        counter: Mutex::new(0),
    });

    let users_scope = web::scope("/users").service(show_users);
    let app_scope = web::scope("/app")
        .route("/index.html", web::get().to(app));

    let www_guard = web::scope("/")
        .guard(guard::Header("Host", "www.rust-lang.org"))
        .route("", web::to(|| async { HttpResponse::Ok().body("www") }));
    let user_guard = web::scope("/")
        .guard(guard::Header("Host", "users.rust-lang.org"))
        .route("", web::to(|| async { HttpResponse::Ok().body("user") }));

    config.app_data(counter);
    config.service(www_guard);
    config.service(user_guard);
    config.service(index);
    config.service(hello);
    config.service(echo);
    config.service(users_scope);
    config.service(app_scope);
    config.route("/hey", web::get().to(manual_hello));
    config.service(
        web::resource("/app1")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app1") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed))
    );
    config.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}
