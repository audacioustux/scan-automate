use std::error::Error;

use api::{config::CONFIG, errors::AppError, serve};
use axum::{
    extract::Path,
    http::{header, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use lettre::{
    message::{header::ContentType, MessageBuilder},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/scans", post(scans_post))
        .route("/scans/confirm/:token", get(scans_confirm))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_headers(vec![header::CONTENT_TYPE])
                .allow_methods([Method::GET]),
        );
    serve(app, CONFIG.port).await;
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
