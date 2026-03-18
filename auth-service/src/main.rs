mod handlers;
mod models;
mod utils;
mod services;

use axum::{
    routing::{ post, get },
    Router
};
use tracing_subscriber;
use dotenvy::dotenv;
use std::{env};
use sqlx::PgPool;

use crate::handlers::{
    auth::{register, login, metrics_handler, resend_email_verification_token, verify_email},
};
use crate::services::auth::AuthService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let db_pool = PgPool::connect(&db_url).await?;

    let auth_service = AuthService::new(db_pool);

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify-email", post(verify_email))
        .route("/resend/email", post(resend_email_verification_token))
        .route("/metrics", get(metrics_handler))
        .with_state(auth_service);

    let addr = "0.0.0.0:3000";
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

