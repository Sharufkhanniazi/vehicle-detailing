mod utils;
mod state;
mod services;
mod handlers;
mod middleware;
mod proto;

use axum::{
    routing::{post, get},
    Router,
    middleware::from_fn_with_state
};
use tokio::task;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use sqlx::PgPool;
use crate::state::AppState;
use crate::services::consumer;
use crate::services::booking::BookingService;
use crate::{handlers::booking::{cancel_booking, metrics_handler, create_booking, get_price, order_completed, submit_review_handler}, middleware::rate_limiting::rate_limit_middleware};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::fmt::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let jwt_secret = env::var("JWT_SECRET_KEY")
        .expect("Failed to read JWT_SECRET_KEY from .env");

    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set");

    let redis = redis::Client::open(redis_url)?;

    let db_pool = PgPool::connect(&db_url).await.unwrap();

    let booking_service = Arc::new(BookingService::new(db_pool.clone()).await?);

    let app_state = AppState::new(jwt_secret, booking_service, redis);

    let consumer_pool = db_pool.clone();

    // start kafka consumer in background
    task::spawn(async move {
        consumer::consumer(consumer_pool).await;
    });

    let app = Router::new()
        .route("/price", get(get_price))
        .route("/booking", post(create_booking))
        .route("/cancel", post(cancel_booking))
        .route("/completed", post(order_completed))
        .route("/review", post(submit_review_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(app_state.clone())
        .layer(from_fn_with_state(
            app_state.clone(), 
            rate_limit_middleware,
        ));

    let addr = "0.0.0.0:3001";
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}