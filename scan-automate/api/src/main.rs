use std::net::SocketAddr;

use axum::{
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let frontend = async {
        let app = Router::new().route("/", get(html));
        serve(app, 3000).await;
    };

    let backend = async {
        let app = Router::new().route("/scan", post(scan)).layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_headers(vec![header::CONTENT_TYPE])
                .allow_methods([Method::GET]),
        );
        serve(app, 4000).await;
    };

    tokio::join!(frontend, backend);
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => TcpListener::from_std(listener).unwrap(),
        None => TcpListener::bind(addr).await.unwrap(),
    };

    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn html() -> impl IntoResponse {
    Html(
        r#"
        check console
        <script>
            fetch('http://localhost:4000/scan', {
                method: "post",
                headers: {
                  'Accept': 'application/json',
                  'Content-Type': 'application/json'
                },

                body: JSON.stringify({
                  url: "https://audacioustux.com",
                  email: "tangimhossain1@gmail.com"
                })
            })
              .then(response => response.json())
              .then(data => console.log(data));
        </script>
        "#,
    )
}

#[derive(Debug, Deserialize, Serialize)]
struct AddScanRequest {
    url: String,
    email: String,
}

async fn scan(Json(req): Json<AddScanRequest>) -> impl IntoResponse {
    dbg!(&req);
    (StatusCode::CREATED, Json(req))
}
