# タイプセーフな情報抽出

> https://actix.rs/docs/extractors

Actix Webは、型安全なリクエスト情報アクセスのための機能として、エクストラクタ(`impl FromRequest`)を提供します。

エクストラクタは、ハンドラ関数の引数としてアクセスできます。
Actix Webはハンドラ関数ごとに最大12個のエクストラクタしています。

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct MyInfo {
    pub id: u32,
    pub username: String,
}

#[get("/extractors")]
pub async fn extractors(path: web::Path<(String, String)>, json: web::Json<MyInfo>) -> impl Responder {
    let path = path.into_inner();
    format!("{} {} {} {}", path.0, path.1, json.id, json.username)
}
```

# パス

パスは、リクエストのパスから抽出された情報を提供します。
抽出可能なパスの部分は動的セグメントと呼ばれ、中カッコで囲まれています。
パスから任意の動的セグメントをデシリアライズすることができます。

例として、`/users/{user_id}/{friend}`というパスに対して登録されたリソースでは、`user_id, friend`という2つのセグメントをデシリアライズすることができます。
これらのセグメントは、宣言された順にタプルとして抽出されます。

```rust
#[get("/users/{user_id}/{friend}")]
async fn user_friend(path: web::Path<(u32, String)>) -> Result<String> {
    let (user_id, friend) = path.into_inner();
    Ok(format!("welcome {}, user_id: {}", friend, user_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(user_friend))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

また、動的セグメント名とフィールド名をマッチングさせることで、`serde`から`Deserialize`トレイトを実装した型へのパス情報を抽出することも可能です。

```rust
#[derive(Deserialize)]
pub struct PostInfo {
    pub post_id: u32,
    pub friend: String,
}

#[get("/posts/{post_id}/{friend}")]
async fn post_friend(info: web::Path<PostInfo>) -> Result<String> {
    Ok(format!(
        "Welcome {}, user_id: {}",
        info.friend, info.post_id
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(post_friend))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

タイプセーフでない代替として、ハンドラ内でパスパラメータの名前でリクエストを問い合わせることもできます。

```rust
#[get("/posts/{post_id}/{friend}")]
async fn post_friend(req: HttpRequest) -> Result<String> {
    let name: String = req.match_info().get("friend").unwrap().parse().unwrap();
    let postid: i32 = req.match_info().query("post_id").parse().unwrap();

    Ok(format!("Welcome {}, post_id: {}", name, postid))
}
```

## クエリ

`Query<T>`型は、リクエストのクエリパラメータを抽出する機能を提供します。
またその下には`serde_urlencoded`クレートが使用されています。

```rust
#[derive(Deserialize)]
struct QueryStruct {
    name: String,
}

#[get("/query")]
async fn query(info: web::Query<QueryStruct>) -> String {
    format!("Welcome {}", info.name)
}
```

以下のコマンドを実行して動作を確認してみましょう。

```bash
curl http://localhost:8080/query?name=ittokun
```

## JSON

`JSON<T>`は、リクエストボディを構造体にデシリアライズすることができます。
リクエストのボディから型付き情報を取り出すには、`T`が`serde::Deserialize`を実装している必要があります。

```rust
#[derive(Deserialize)]
struct JsonStruct {
    name: String,
}

#[post("/json")]
async fn json(info: web::Json<JsonStruct>) -> Result<String> {
    Ok(format!("Welcome {}", info.name))
}
```

以下のコマンドを実行して動作を確認してみましょう。

```bash
curl -k https://localhost:8080/json -X POST -H "Content-Type: application/json" -d '{"name":"ittokun"}'
```

抽出機能の中には、抽出機能を設定する方法を提供しているものがあります。
抽出機を設定するには、`.add_data()`メソッドにその設定オブジェクトを渡します。
JSON抽出器の場合、`JsonConfig`が返されます。JSONペイロードの最大サイズや、カスタムエラーハンドラ関数など設定することが可能です。

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().finish())
                    .into()
            });

        App::new().service(
            web::resource("/")
                .app_data(json_config)
        )
}
```

## URLエンコードフォーム

URLエンコードされたフォームボディは、`JSON<T>`のような構造体に抽出することができます。

```rust
#[post("/form")]
async fn form(form: web::Form<FormData>) -> Result<String> {
    Ok(format!("Welcome {}", form.username))
}
```

以下のコマンドを実行して動作を確認してみましょう。

```bash
curl -X POST http://localhost:8080/form  -d "username=ittokun"
```

## その他

Actix Webには他にも多くの抽出機を提供しています。

- `Data`、アプリケーションの状態の断片にアクセスする
- `HttpRequest`、HTTPRequest自体が抽出器であり、リクエストの他の部分にアクセスする時に使用
- `String`、リクエストのペイロードをStringに変換することができる
- `Bytes`、リクエストのペイロードをBytesに変換することができる
- `Payload`、主に他の抽出機を構築するための低レベルのペイロード抽出器

## アプリ状態抽出

アプリケーションの状態は、ハンドラから`web::Data`エクストラクタでアクセスできます。
しかし状態は読み取り専用の参照としてアクセスできます。
もし`state`に`mutable`なアクセスが必要な場合は、それを実装する必要があります。

```rust
#[get("/count")]
async fn show_count(data: web::Data<StateStruct>) -> impl Responder {
    format!("count: {}", data.count.get())
}

#[get("/add-one")]
async fn add_one(data: web::Data<StateStruct>) -> impl Responder {
    let count = data.count.get();
    data.count.set(count + 1);

    format!("Count: {}", data.count.get())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app = move || {
        let state_counter = web::Data::new(api::StateStruct {
            count: Cell::new(0),
        });

        App::new()
            .app_data(state_counter)
            .service(api::show_count)
            .service(api::add_one)
    };

    HttpServer::new(app).bind(("127.0.0.1", 8080))?.run().await
}
```

すべてのスレッドで状態を共有したい場合は、`web::Data`と`app_data`を使用します。

MutexやRwLockなどのブロッキング同期プリミティブをアプリのステートで使用する場合は、注意が必要です。
Actix Webはリクエストを非同期で処理します。ハンドラ内のクリティカルセクションが大きすぎたり、`await`ポイントが含まれていたりすると問題です。
これが気になる場合はTokioのアドバイス「[非同期コードでブロッキングMutexを使う](https://tokio.rs/tokio/tutorial/shared-state#on-using-stdsyncmutex) 」を読むことをお勧めします。
