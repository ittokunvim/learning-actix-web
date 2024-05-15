# HTTPサーバー

> Ref: https://actix.rs/docs/server

`HTTPServer`型は、HTTPリクエストの処理を行います。

`HTTPServer`はアプリケーションファクトリをパラメータとして受け取り、これには`Send + Sync`の境界を持つ必要があります。
これらについてはマルチスレッドのセクションで詳しく説明します。

ウェブサーバーを起動するには、まずネットワークソケットにバインドする必要があります。
`HttpServer::bind()`に、ソケットアドレスのタプル、文字列("127.0.0.1", "0.0.0.0:8080")などの値を指定して実行します。
ソケットが他のアプリケーションによって使用されている場合失敗します。

バインドが成功したら、`HttpServer::run()`を使用して`Server`のインスタンスを返します。
`Server`は、リクエストの処理を開始するために待機、または起動する必要があり、シャットダウン信号（デフォルトでは`Ctrl-c`など）を受け取るまで実行されます。

```rust
use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(HttpResponse::Ok)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

## マルチスレッド

`HttpServer`は自動的にいくつかのHTTPワーカーを起動します。
デフォルトの数は、システムの物理CPUの数に等しくなっています。
この数は、`HttpServer::workers()`メソッドでオーバーライドすることができます。

```rust
#[actix_web::main]
async fn main() {
    HttpServer::new(|| App::new().route("/", web::get().to(HttpResponse::Ok))).workers(4);
}
```

ワーカーが作成されると、それぞれ個別のアプリケーションインスタンスを受け取り、リクエストを処理します。
アプリケーションの状態はスレッド間で共有されず、ハンドラは状態のコピーを自由に操作でき、並行性の心配はありません。

アプリケーションの状態は`Send, Sync`である必要はありませんが、アプリケーションファクトリは`Send, Sync`である必要があります。

ワーカースレッド間で状態を共有するには、`Arc/Data`を使用します。
共有と動機が導入されたら、特別な注意を払う必要があります。
多くの場合、修正のために共有状態をロックする結果、不注意にパフォーマンスコストが発生します。

場合によっては、効率的なロック戦略として、たとえばミューテックスの代わりに読み書きロックを使用して非排他的ロックを実現することでこれらのコストを軽減することができます。
しかし最もパフォーマンスの高い実装は、ロックを全く発生させないものになることが多いようです。

各ワーカースレッドはリクエストを順番に処理するので、現在のスレッドをブロックするハンドラは、現在のワーカーが新しいリクエストの処理を停止する原因となります。

このため、`I/O`やデータベース操作など、CPUに縛られない長い操作は、`features`や非同期関数として表現する必要があります。
非同期ハンドラはワーカースレッドによって同時に実行されるため、実行がブロックされることはありません。

```rust
async fn my_handler() -> impl Responder {
    tokio::time::sleep(Duration::from_secs(5)).await;
	"response"
}
```

同じ制限が抽出器にも適用されます。
ハンドラ関数が`FromRequest`を実装した引数を受け取り、その実装が現在のスレッドをブロックする場合、ワーカースレッドはハンドラを実行する際にブロックされます。
そのためエクストラクタを実装する際には注意が必要で、必要な場合は非同期に実装する必要があります。

## TLS/HTTPS

Actix Webは、`rustls, openssl`の2つのTLS実装をそのままサポートしています。

`rustls`クレートは`rustls`との統合、`openssl`は`openssl`との統合のためです。

```toml
[dependencies]
actix-web = { version = "4.3.0", features = ["openssl"] }
openssl = "0.10.45"
```

```rust
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
    "Welcome!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| App::new().service(index))
        .bind_openssl(("127.0.0.1", 8080), builder)?
        .run()
        .await
}
```

以下のコマンドを実行して、`key.pem, cert.pem`を作成します。自分の好きな科目を記入する

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
-days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"
```

パスワードを消すには、`nopass.pem`を`key.pem`にコピーします。

```bash
openssl rsa -in key.pem -out nopass.pem
```

実装が終わったら、サーバーにアクセスしてみましょう。`-k`オプションはセキュリティを無視します。
使用には注意しましょう。

```bash
curl -k https://localhost:8080/
```

## キープアライブ

Actix Webは、後続のリクエストを待つためにコネクションを開いたままにします。

> キープアライブ接続の動作は、サーバーの設定によって定義されます。

1. `Duration::from_secs(75), KeepAlive::Timeout(75)`、75秒間のキープアライブタイマーを有効
2. `KeepAlive::Os`、OSのキープアライブを使用
3. `None, KeepAlive::Disabled`、キープアライブを無効

```rust
use actix_web::{http::KeepAlive, HttpServer};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _one   = HttpServer::new(app).keep_alive(Duration::from_secs(75));
    let _two   = HttpServer::new(app).keep_alive(KeepAlive::Os);
    let _three = HttpServer::new(app).keep_alive(None);
    Ok(())
}
```

上記の最初のオプションが選択された場合、HTTP/1.1リクエストではレスポンスが接続タイプをCloseやUpgradeに設定するなどして明示的に拒否していなければ、`keep-alive`が有効になります。
接続を強制的に閉じるには、`HttpResponseBuilder`の`force_close()`メソッドを使用します。

```rust
pub async fn quit() -> HttpResponse {
    HttpResponse::Ok()
        // .connection_type(http::ConnectionType::Close)
        .force_close()
        .finish()
}
```

## シャットダウン

`HttpServer`は、*Graceful shutdown*をサポートしています。
停止シグナルを受け取った後、ワーカーにはリクエスの処理を終えるまでに、決められた時間があります。
タイムアウト後にまだ生存しているワーカーは強制的にシャットダウンされます。
デフォルトでは、シャットダウンのタイムアウトは30秒に設定されています。
このパラメータは、`HttpServer::shutdown_timeout()`メソッドで変更することができます。

`HttpServer`は、いくつかのOSシグナルを扱います。`Ctrl-C`は全てのOSで利用可能で、その他のシグナルはunixシステムで利用可能です。

- SIGINT、強制シャットダウン
- SIGTERM、グレースフルシャットダウン
- SIGQUIT、強制シャットダウン

> `HttpServer::disable_signals()`メソッドで、シグナル処理を無効にすることができます。
