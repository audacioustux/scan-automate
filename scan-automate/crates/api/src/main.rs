use std::error::Error;

use api::{config::CONFIG, errors::AppError, serve};
use axum::{
    extract::{Path, State},
    http::StatusCode,
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
use nanoid::nanoid;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let app = Router::new()
        .route("/scans", post(scans_post))
        .route("/scans/confirm/:token", get(scans_confirm))
        .route("/scans/progress/:id", get(scans_progress))
        .layer(CorsLayer::permissive())
        .with_state(client);

    serve(app, CONFIG.port).await;
}

#[derive(Debug, Deserialize, Serialize)]
struct RustscanConfig {
    uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ZapConfig {
    uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScanID(String);

#[derive(Debug, Deserialize, Serialize)]
struct ScanRequest {
    email: String,
    rustscan: Option<RustscanConfig>,
    zap: Option<ZapConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScanRequestWithID {
    id: ScanID,
    req: ScanRequest,
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

fn create_scan_request_token(req: &ScanRequestWithID) -> Result<String, impl Error> {
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

async fn scans_post(Json(req): Json<ScanRequest>) -> ApiResponse<ScanID> {
    let id = {
        let valid_pod_name_chars: Vec<char> = ('a'..='z').chain('0'..='9').collect();
        ScanID(nanoid!(10, &valid_pod_name_chars))
    };

    let instance = ScanRequestWithID { id, req };

    let token = create_scan_request_token(&instance)?;
    let body = format!(
        r#"
        Hi,

        Please confirm your scan request by clicking the link below:

        http://localhost:4000/scans/confirm/{}
        "#,
        token
    );

    let email = Message::builder()
        .from(CONFIG.smtp_from.parse()?)
        .to(instance.req.email.parse()?)
        .subject("Confirm Scan Request")
        .header(ContentType::TEXT_PLAIN)
        .body(body)?;

    get_mailer().send(&email)?;

    Ok((StatusCode::OK, Json(instance.id)))
}

async fn scans_confirm(
    Path(token): Path<String>,
    State(client): State<Client>,
) -> ApiResponse<Value> {
    let TokenData { claims, .. } = decode::<JWTClaims<ScanRequestWithID>>(
        &token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    client
        .post(&CONFIG.scan_webhook_url)
        .body(serde_json::to_string(&claims.data)?)
        .send()
        .await?;

    Ok((StatusCode::OK, Json(json!("ok"))))
}

async fn scans_progress(
    Path(id): Path<String>,
    State(client): State<Client>,
) -> ApiResponse<Value> {
    let res: serde_json::Value = client
        .get(&format!(
            "https://{}/api/v1/workflows/argo/scan-{}",
            CONFIG.argo_workflow_host, id
        ))
        .header("Authorization", &CONFIG.argo_workflow_token)
        .send()
        .await?
        .json()
        .await?;

    Ok((StatusCode::OK, Json(res)))
}

type ApiResponse<T> = Result<(StatusCode, Json<T>), AppError>;
