# エラー

> Ref: https://actix.rs/docs/errors

Actix Webは、Webハンドラからのエラー処理に、独自の`actix_web::error::Error`タイプと、`actix_web::error::ResponseError`トレイトを使用しています。

ハンドラが`ResponseError`を実装した`Result`で`Error`を返す場合、actix-webはそのエラーをHTTP応答として、対応する`actix_web::http::StatusCode`でレンダーします。
デフォルトでは、内部サーバーエラーが生成されます。（以下参照）

```rust
pub trait ResponseError {
    fn error_response(&self) -> Response<Body>;
    fn status_code(&self) -> StatusCode;
}
```

`Responder`は、互換性のある結果をHTTPレスポンスに変換します。

```rust
impl<T: Responder, E: Into<Error>> Responder for Result<T, E>
```

上記のコードの`Error`はactix-webのエラー定義であり、`ResponseError`を実装したエラーは自動的に変換することができます。

Actix Webは、いくつかの一般的な非actixエラーに対する`ResponseError`の実装を提供します。
例えば、ハンドラーが`io::Error`で応答した場合、そのエラーは`HttpInternalServerError`に変換されます。

```rust
use std::io;
use actix_files::NamedFile;

fn index(_req: HttpRequest) -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}
```

## カスタムエラーレスポンス

ここでは、`ResponseError`の例として、`derive_more`クレートによる宣言エラー列挙型を使用します。

```rust

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display(fmt = "my error: {}", name)]
struct CustomError {
    name: &'static str,
}

impl actix_web::error::ResponseError for CustomError {}

#[get("custom-error")]
async fn custom_error() -> Result<&'static str, CustomError> {
    Err(CustomError { name: "test" })
}
```

`ResponseError`は、`error_response()`のデフォルト実装で500をレンダリングするようになっています。
上記のインデックスハンドラが実行されるとこのような状態になります。

`error_response()`をオーバーライドして、より有用な結果を得ることができます。

```rust
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
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            CustomErrorEnum::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            CustomErrorEnum::BadClientData => StatusCode::BAD_REQUEST,
            CustomErrorEnum::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[get("custom-error-enum")]
async fn custom_error_enum() -> Result<&'static str, CustomErrorEnum> {
    let internal_error = Err(CustomErrorEnum::InternalError)?;
    let _bad_client_data = Err(CustomErrorEnum::BadClientData)?;
    let _timeout = Err(CustomErrorEnum::Timeout)?;

    internal_error
}
```

## エラーヘルパー

Actix-webは、他のエラーから特定のHTTPエラーコードを生成するのに便利なエラーヘルパー関数のセットを持っています。
ここでは、`map_err`を使用して、`ResponseError`トレイトを実装していない`CustomError`を400に変換します。

```rust
#[derive(Debug)]
struct CustomError {
    name: &'static str,
}

#[get("/map-err")]
async fn map_err() -> Result<&'static str> {
    let result: Result<&'static str, CustomError> = Err(CustomError { name: "test error" });
    Ok(result.map_err(|e| error::ErrorBadRequest(e.name))?)
}
```

## エラーログ1

Actix-webは、全てのエラーを`WARN`ログレベルでログに記録します。
アプリケーションのログレベルが`DEBUG`に設定され、`RUST-BACKTRACE`が有効になっている場合、バックトレースもログに記録されます。

```bash
RUST_BACKTRACE=1 RUST_LOG=actix_web=debug cargo run
```

`Error`タイプは利用可能な場合、原因のエラーバックトレースを使用します。
基礎となる障害がバックトレースを提供しない場合、新しいバックトレースは変換に発生したポイントを差して構築されます。

## エラー処理の推奨事項

アプリケーションが発生させるエラーは2つの大きなグループに分けて考えると良いでしょう。
1つ目はユーザ向けのエラーと、2つ目はそれ以外のものです。

前者の例としては、ユーザーが不正な入力をしたときに`ValidationError`をカプセル化した`UserError enum`を`failure`で指定することです。

`display`で定義されたエラーメッセージは、ユーザーが読むことを明確に意識して書かれています。

しかし全てのエラーに対してメッセージを送り返すことは望ましいことではありません。
例として以下のようなものがあります。

- データベースがダウンしてクライアントライブラリが接続タイムアウトエラー
- HTMLテンプレートが不適切にフォーマットされてレンダリング時にエラーが発生するケース

このような場合、エラーを一時的なエラーにマッピングして、ユーザーが利用できるようにすることが望ましいかもしれません。

エラーをユーザーと向き合うものとそうでないものに分けることで、アプリケーション内部で発生するユーザーが見るはずのないエラーに誤って晒されることがないようにすることができるのです。

## エラーログ2

以下の例は、`env_logger, log`に依存する`middleware::Logger`を使った基本的なコードです。

```rust
use actix_web::middleware::Logger;
use log::info;

#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
pub struct MyError {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyError {}

#[get("/")]
async fn index() -> Result<&'static str, MyError> {
    let err = MyError { name: "test error" };
    info!("{}", err);
    Err(err)
}

#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        let logger = Logger::default();

        App::new()
          .wrap(logger)
          .service(index)
    })
    .bind_openssl(("127.0.0.1", 8080), builder)?
    .run()
    .await
}
```
