use api::serve;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(html));
    serve(app, 3000).await;
}

async fn html() -> impl IntoResponse {
    Html(include_str!("index.html"))
}
