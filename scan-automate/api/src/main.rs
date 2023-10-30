mod config;
mod errors;

use std::{error::Error, net::SocketAddr};

use axum::{
    extract::Path,
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use config::{Config, CONFIG};
use errors::AppError;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lettre::{
    message::{header::ContentType, MessageBuilder},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    Config::parse();

    let frontend = async {
        let app = Router::new().route("/", get(html));
        serve(app, 3000).await;
    };

    let backend = async {
        let app = Router::new()
            .route("/scans", post(scans_post))
            .route("/scans/confirm/:token", get(scans_confirm))
            .layer(
                CorsLayer::new()
                    .allow_origin("*".parse::<HeaderValue>().unwrap())
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

#[derive(Debug, Deserialize, Serialize)]
struct AddScanRequest {
    url: String,
    email: String,
}

fn get_mailer() -> SmtpTransport {
    let smtp_host = &CONFIG.smtp_host;
    let smtp_username = &CONFIG.smtp_username;
    let smtp_password = &CONFIG.smtp_password;

    let creds = Credentials::new(smtp_username.into(), smtp_password.into());

    SmtpTransport::relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .build()
}

fn get_mail_builder() -> MessageBuilder {
    let from = &CONFIG.email_from;

    Message::builder()
        .from(from.parse().unwrap())
        .reply_to(from.parse().unwrap())
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanRequestClaims {
    url: String,
    sub: String,
    exp: i64,
}

fn create_scan_request_token(url: &str, sub: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(1))
        .expect("valid timestamp")
        .timestamp();

    let claims = ScanRequestClaims {
        url: url.into(),
        sub: sub.into(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
    )
}

fn scan_request_email(token: &str) -> Result<Message, impl Error> {
    let template = |token: &str| {
        format!(
            r#"
            Hi,

            Please confirm your scan request by clicking the link below:

            http://localhost:4000/scans/confirm/{}
            "#,
            token
        )
    };

    get_mail_builder()
        .to("Tanjim Hossain <tanjimhossain.pro@gmail.com>"
            .parse()
            .unwrap())
        .subject("Confirm Scan Request")
        .header(ContentType::TEXT_PLAIN)
        .body(template(token))
}

async fn scans_post(Json(req): Json<AddScanRequest>) -> Result<impl IntoResponse, AppError> {
    let AddScanRequest { url, email } = req;

    let token = create_scan_request_token(&url, &email)?;
    let email = scan_request_email(&token)?;
    get_mailer().send(&email)?;

    Ok((StatusCode::OK, Json(json!({ "status": "ok" }))))
}

fn validate_scan_request_token(token: &str) -> Result<TokenData<ScanRequestClaims>, impl Error> {
    let validation = Validation::new(Algorithm::HS256);

    decode::<ScanRequestClaims>(
        &token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
        &validation,
    )
}

async fn scans_confirm(Path(token): Path<String>) -> Result<impl IntoResponse, AppError> {
    let claims = validate_scan_request_token(&token)?;

    dbg!(claims);

    Ok((StatusCode::OK, Json(json!({ "status": "ok" }))))
}
