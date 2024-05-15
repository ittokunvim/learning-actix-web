use actix_web::{get, web, http, Error, HttpRequest, HttpResponse};
use serde::{Serialize, Deserialize};
use futures::stream;

use std::task::Poll;

#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub counter: i32,
}

async fn index(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body(format!("hello: {}", req.path()))
}

#[get("testing/app-data")]
async fn app_state(data: web::Data<AppState>) -> HttpResponse {
    let mut app = AppState {
        counter: data.counter,
    };
    app.counter += 1;

    HttpResponse::Ok().json(app)
}

#[get("testing/stream")]
async fn sse() -> HttpResponse {
    let mut counter: usize = 5;

    let server_events = stream::poll_fn(move |_| -> Poll<Option<Result<web::Bytes, Error>>> {
        if counter == 0 {
            return Poll::Ready(None);
        }
        let payload = format!("data: {}\n\n", counter);
        counter -= 1;
        Poll::Ready(Some(Ok(web::Bytes::from(payload))))
    });

    HttpResponse::build(http::StatusCode::OK)
        .insert_header((http::header::CONTENT_TYPE, "text/event-stream"))
        .insert_header(http::header::ContentEncoding::Identity)
        .streaming(server_events)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let counter = web::Data::new(AppState {
        counter: 3,
    });

    cfg.route("/testing", web::get().to(index));
    cfg.app_data(counter);
    cfg.service(app_state);
    cfg.service(sse);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, body, body::MessageBody as _, rt::pin, App};
    use std::future;

    #[actix_web::test]
    async fn test_index_ok() {
        let req = test::TestRequest::default()
            .insert_header(http::header::ContentType::plaintext())
            .to_http_request();
        let res = index(req).await;
        assert_eq!(res.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_index_not_ok() {
        let req = test::TestRequest::default().to_http_request();
        let res = index(req).await;
        assert_ne!(res.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().configure(init_routes)).await;
        let req = test::TestRequest::get()
            .uri("/testing")
            .insert_header(http::header::ContentType::plaintext())
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_index_post() {
        let app = test::init_service(App::new().configure(init_routes)).await;
        let req = test::TestRequest::post()
            .uri("/testing")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_index_app_data() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { counter: 4 }))
                .configure(init_routes)
        ).await;
        let req = test::TestRequest::get()
            .uri("/testing/app-data")
            .to_request();
        let res: AppState = test::call_and_read_body_json(&app, req).await;

        assert_eq!(res.counter, 4);
    }

    #[actix_web::test]
    async fn test_stream_chunk() {
        let app = test::init_service(App::new().configure(init_routes)).await;
        let req = test::TestRequest::get()
            .uri("/testing/stream")
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = res.into_body();
        pin!(body);

        // first chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 5\n\n")
        );

        // second chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 4\n\n")
        );

        // remaining part
        for i in 0..3 {
            let expected_data = format!("data: {}\n\n", 3 - i);
            let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
            assert_eq!(bytes.unwrap().unwrap(), web::Bytes::from(expected_data));
        }
    }

    #[actix_web::test]
    async fn test_stream_full_payload() {
        let app = test::init_service(App::new().configure(init_routes)).await;
        let req = test::TestRequest::get()
            .uri("/testing/stream")
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let body = res.into_body();
        let bytes = body::to_bytes(body).await;
        assert_eq!(
            bytes.unwrap(),
            web::Bytes::from_static(b"data: 5\n\ndata: 4\n\ndata: 3\n\ndata: 2\n\ndata: 1\n\n")
        );
    }
}
