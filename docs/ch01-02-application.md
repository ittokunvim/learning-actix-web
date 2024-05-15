# アプリケーションの作成

> Ref: https://actix.rs/docs/application

`actix-web`は、Rustでウェブサーバーやアプリケーションを構築するための様々なプリミティブを提供します。
ルーティング、ミドルウェア、リクエストの前処理、レスポンスの後処理などです。

全ての`actix-web`サーバーは、Appインスタンスを中心に構築されています。
これは、リソースやミドルウェアのルートを登録するために使用されます。
また同じスコープ内のすべてのハンドラで共有されるアプリケーションの状態も保存されます。

アプリケーションのスコープは、すべてのルートの名前空間として機能します。
つまり特定のアプリケーションスコープのすべてのルートは、同じURLパスのプレフィックスを持ちます。
アプリケーションのプレフィックスは、常に先頭のスラッシュを含んでいます。
提供されたプレフィクスが先頭のスラッシュを含んでいない場合、自動的に挿入されます。
プレフィクスは値のパスセグメントで構成されている必要があります。

```rust
async fn index() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/app")
                    .route("/index.html", web::get().to(index)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

この例では、`/app`という接頭子を持つアプリケーションと`index.html`というリソースを作成します。以下のコマンドで動作を確認してみましょう。

```bash
curl http://localhost:8080/app/index.html
```

## 状態

アプリケーションの状態は、同じスコープ内のすべてのルートとリソースで共有されます。
状態は`web::Data<T>`を使ってアクセスします。またミドルウェアからでもアクセス可能です。

```rust
struct AppState {
    app_name: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Hello {app_name}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

以下のコマンドを実行して、"Hello Actix Web"と出力されるか確認してみましょう。

```bash
curl http://localhost:8080/
```

## 共有変数状態

`HttpServer`は、アプリケーションインスタンスではなく、アプリケーションファクトリを受け付けます。
`HttpServer`は、各スレッドに対してアプリケーションのインスタンスを構築します。
そのためアプリケーションのデータを複数回構築する必要があります。
異なるスレッド間でデータを共有したい場合は、`Send + Sync`のような共有可能なオブジェクトを使用する必要があります。

内部的には、`web::Data`は`Arc`を使用しています。
そのため2つ以上のArcを作らないように、`App::app_data()`を使って登録する前にデータを作成する必要があります。

```rust
struct AppStateWithCounter {
    counter: Mutex<i32>,
}

#[get("/")]
async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Hello {app_name}, Request number: {counter}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        app_name: String::from("Actix Web"),
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

主な点

- `HttpServer::new`に渡されたクロージャの内部で初期化された状態は、ローカルのワーカースレッドにあり、変更すると同期が解除される可能性がある。
- グローバルに共有される状態を実現するには、`HttpServer::new`に渡されたクロージャの外部で状態を作成し、移動とクローンをする必要があります。

## スコープを使ったアプリケーションの構成

`web::scope()`メソッドは、リソースグループのプレフィクスを設定することが出来ます。
このスコープは、リソース設定によって追加されるすべてのリソースパターンの前に付加されるプレフィクスを表します。
これを使用すると、同じリソース名を維持したまま、一連のルートをマウントするのに役立ちます。

```rust
#[actix_web::main]
async fn main() {
    let scope = web::scope("/users").service(show_users);
    App::new().service(scope);
}
```

上記の例では、`show_users`ルートは`/show`ではなく`/users/show`という有効なルートパターンを持ちます。
これは、アプリケーションのスコープ引数がパターンの前に追加されるからです。
そしてルートはURLパスが`/users/show`の場合にのみマッチし、ルート名`show_users`で`HttpRequest.url_for()`関数が呼ばれると、その同じパスのURLが生成されることになります。

## アプリガードとバーチャルホスティング

ガードは、リクエストオブジェクトの参照を受け取り、`true, false`を返す単純な関数です。
形式的にガードは、`Guard`トレイトを実装した任意のオブジェクトです。

以下の例は、ガードで提供されているHeaderを使って、リクエストのヘッダ情報に基づいたフィルタを行っています。

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/")
                    .guard(guard::Header("Host", "www.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("www") })),
            )
            .service(
                web::scope("/")
                    .guard(guard::Header("Host", "users.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("user") })),
            )
    })
}
```

## 設定

シンプルさと再利用性のために、`App, web::Scope`には`configure`メソッドがあります。
この関数は、設定の一部を別のモジュール、あるいはライブラリに移動させるのに便利です。
例えば、リソースの設定の一部を別のモジュールに移動させることが出来ます。

```rust
fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
        .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed))
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	HttpServer::new(move || {
        App::new()
            .configure(config)
            .service(web::scope("/api").configure(scoped_config))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

以下のコマンドを入力して、出力を確認してみましょう。

```bash
curl http://localhost:8080/app
curl http://localhost:8080/api/test
```
