use std::error::Error;

use api::{config::CONFIG, errors::AppError, serve};
use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let app = Router::new()
        .route("/scans", post(scans_post))
        .route("/scans/confirm/:token", get(scans_confirm))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_headers(vec![header::CONTENT_TYPE])
                .allow_methods([Method::GET]),
        )
        .with_state(client);

    serve(app, CONFIG.port).await;
}

#[derive(Debug, Deserialize, Serialize)]
struct ScanRequest {
    url: String,
    email: String,
    rustscan: bool,
    zap: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JWTClaims<T> {
    data: T,
    exp: i64,
}

fn create_scan_request_token(req: &ScanRequest) -> Result<String, impl Error> {
    let exp = Utc::now()
        .checked_add_signed(Duration::days(1))
        .unwrap()
        .timestamp();

    let claims = JWTClaims { data: req, exp };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
    )
}

async fn scans_post(Json(req): Json<ScanRequest>) -> Result<impl IntoResponse, AppError> {
    let token = create_scan_request_token(&req)?;
    let body = format!(
        r#"
            Hi,

            Please confirm your scan request by clicking the link below:

            http://localhost:4000/scans/confirm/{}
            "#,
        token
    );

    let email = Message::builder()
        .from(CONFIG.email_from.parse()?)
        .to(req.email.parse()?)
        .subject("Confirm Scan Request")
        .header(ContentType::TEXT_PLAIN)
        .body(body)?;

    get_mailer().send(&email)?;

    Ok((StatusCode::OK, Json(json!({ "status": "ok" }))))
}

fn validate_scan_request_token(
    token: &str,
) -> Result<TokenData<JWTClaims<ScanRequest>>, impl Error> {
    let validation = Validation::new(Algorithm::HS256);

    decode(
        &token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
        &validation,
    )
}

async fn scans_confirm(
    Path(token): Path<String>,
    State(client): State<Client>,
) -> Result<impl IntoResponse, AppError> {
    let TokenData { claims, .. } = validate_scan_request_token(&token)?;

    client
        .post(&CONFIG.scan_webhook_url)
        .body(serde_json::to_string(&claims.data)?)
        .send()
        .await?;

    Ok((StatusCode::OK, Json(json!({ "status": "ok" }))))
}
