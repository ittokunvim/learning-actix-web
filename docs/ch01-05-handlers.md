# リクエストハンドラ

> Ref: https://actix.rs/docs/handlers

リクエストハンドラは、リクエストから抽出できる0個以上のパラメータを受け取り、`HttpResponse`に変換できる型を返す非同期関数です。

リクエスト処理は2段階で行われます。
まず、ハンドラオブジェクトが呼び出され、レスポンス特性を実装した任意のオブジェクトを返します。
次に、返されたオブジェクトに対して`respond_to()`が呼び出され、それ自体が`HttpResponse, Error`に変換されます。

Actix Webのデフォルトでは、`&'static str, String`などの標準的な型に対して`Responder`の実装を提供しています。

```rust
async fn index(_req: HttpRequest) -> &'static str {
    "Hello World!"
}

async fn index(_req: HttpRequest) -> String {
    "Hello World!".to_owned()
}
```

また、シグネチャを変更して、より複雑な方が含まれる場合に有効な`impl responder`を返すようにすることもできます。

```rust
async fn index(_req: HttpRequest) -> impl Responder {
    web::Bytes::from_static(b"Hello World!")
}

async fn index(_req: HttpRequest) -> Box<Future<Item=HttpResponse, Error=Error>> {
    // ...
}
```

## レスポンスカスタムタイプ

ハンドラ関数からカスタムタイプを直接返すには、そのタイプに`Responder`トレイトを実装する必要があります。

カスタムタイプのレスポンスで、`application/json`レスポンスにシリアライズするものを作成してみましょう。

```rust
#[derive(Serialize)]
struct CustomType {
    name: &'static str,
}

impl Responder for CustomType {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
        .content_type(ContentType::json())
            .body(body)
    }
}

#[get("custom-type")]
async fn custom_type() -> impl Responder {
    CustomType { name: "ittokun" }
}
```

## ストリーミングレスポンス

`body`に`Stream<Item=Bytes, Error=Error>`を実装することで、レスポンスボディを非同期で生成することができます。

```rust
use futures::{future::ok, stream::once};

#[get("/stream")]
async fn stream() -> HttpResponse {
    let body = once(ok::<_, Error>(web::Bytes::from_static(b"test")));

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(body)
}
```

## 異なる返り型

時々、異なるタイプのレスポンスを返したいことがあります。
例えば、エラーチェックをしてエラーを返したり、非同期応答を返したり、2つの異なるタイプを必要とする結果を返したりです。

`Either`タイプを使用すれば、2つの異なるレスポンダタイプを1つのタイプにまとめることができます。

```rust
type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>;

#[get("either")]
async fn either() -> RegisterResult {
    if true {
        Either::Left(HttpResponse::BadRequest().body("Bad data"))
    } else {
        Either::Right(Ok("Hello!"))
    }
}
```
