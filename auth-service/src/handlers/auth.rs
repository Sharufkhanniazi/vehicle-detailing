use axum::{Json, extract::{State, Query}, response::IntoResponse};
use std::time::Instant;
use crate::services::auth::AuthService;
use crate::models::{RegisterUser, LoginUser, EmailVerification, VerifyEmailRequest};
use crate::services::metrics::{
    gather_metrics,
    HTTP_REQUESTS_TOTAL,
    HTTP_REQUEST_DURATION,
    HTTP_RESPONSE_SIZE,
    HTTP_RESPONSE_STATUS
};
use crate::utils::error::{Result};

pub async fn register(
    State(auth_service): State<AuthService>,
    Json(register_data): Json<RegisterUser>,
) -> Result<impl IntoResponse> {

    let timer = Instant::now();
    HTTP_REQUESTS_TOTAL.with_label_values(&["POST", "/register"]).inc();

    let response = auth_service.register(register_data).await?;

    let duration = timer.elapsed().as_secs_f64();
    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/register"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/register", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/register"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn login(
    State(auth_service): State<AuthService>,
    Json(login_data): Json<LoginUser>,
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/login"])
        .inc();

    let response = auth_service.login(login_data).await?;

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/login"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/login", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/login"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn resend_email_verification_token(
    State(auth_service): State<AuthService>,
    Json(verification_data): Json<EmailVerification>,
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/resend/email"])
        .inc();

    let response = auth_service.resend_email_verification_token(&verification_data.email).await?;

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/resend/email"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/resend/email", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/resend/email"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn verify_email(
    State(auth_service): State<AuthService>,
    Query(payload): Query<VerifyEmailRequest>,
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/verify-email"])
        .inc();

    let response = auth_service.verify_email(&payload.token).await?;

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/verify-email"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/verify-email", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
    .with_label_values(&["/verify-email"])
    .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(response)
}

pub async fn metrics_handler() -> impl IntoResponse {
    gather_metrics()
}