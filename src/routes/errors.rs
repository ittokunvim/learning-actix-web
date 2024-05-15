use actix_web::{get, web, http, body, error, Result, HttpResponse};
use actix_files::NamedFile;
use log::info;

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display(fmt = "my error: {}", name)]
struct CustomError {
    name: &'static str,
}

impl error::ResponseError for CustomError {}

#[derive(Debug, derive_more::Display)]
enum CustomErrorEnum {
    #[display(fmt = "internal error")]
    InternalError,
    #[display(fmt = "bad request")]
    BadClientData,
    #[display(fmt = "timeout")]
    Timeout,
}

impl error::ResponseError for CustomErrorEnum {
    fn error_response(&self) -> HttpResponse<body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(http::header::ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> http::StatusCode {
        match *self {
            CustomErrorEnum::InternalError => http::StatusCode::INTERNAL_SERVER_ERROR,
            CustomErrorEnum::BadClientData => http::StatusCode::BAD_REQUEST,
            CustomErrorEnum::Timeout => http::StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[get("/static-index")]
async fn static_index() -> std::io::Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

#[get("/custom-error")]
async fn custom_error() -> Result<&'static str, CustomError> {
    Err(CustomError { name: "test" })
}

#[get("/custom-error-enum")]
async fn custom_error_enum() -> Result<&'static str, CustomErrorEnum> {
    let internal_error = Err(CustomErrorEnum::InternalError)?;
    let _bad_client_data = Err(CustomErrorEnum::BadClientData)?;
    let _timeout = Err(CustomErrorEnum::Timeout)?;

    internal_error
}

#[get("/map-err")]
async fn map_err() -> Result<&'static str> {
    let result: Result<&'static str, CustomError> = Err(CustomError { name: "test error" });
    Ok(result.map_err(|e| error::ErrorBadRequest(e.name))?)
}

#[get("/err-logging")]
async fn err_logging() -> Result<&'static str, CustomError> {
    let err = CustomError { name: "Error Logging" };
    info!("{}", err);
    Err(err)
}

pub fn init_routes(config: &mut web::ServiceConfig) {
  config.service(static_index);
  config.service(custom_error);
  config.service(custom_error_enum);
  config.service(map_err);
  config.service(err_logging);
 }
