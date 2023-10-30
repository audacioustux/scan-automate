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
    Html(
        r#"
        check console
        <script>
            fetch('http://localhost:4000/scans', {
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
