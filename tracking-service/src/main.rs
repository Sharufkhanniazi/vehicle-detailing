mod utils;
mod state;
mod services;
mod handlers;
mod middleware;

use axum::{
    routing::{post, get},
    Router,
    middleware::from_fn_with_state
};
use dotenvy::dotenv;
use std::env;
use sqlx::PgPool;
use tokio::task;
use crate::{middleware::rate_limiting::rate_limit_middleware, state::AppState};
use crate::services::consumer;
use crate::{handlers::tracking::{
    calculate_distance_handler, 
    get_tracking_handler, 
    notify_arrival_handler, 
    update_location_handler, 
    ws_tracking_handler,
    metrics_handler
    },     
    services::kafka_producer::KafkaProducer
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set");

    let jwt_secret = env::var("JWT_SECRET_KEY")
        .expect("Failed to read JWT_SECRET_KEY from .env");

    let broker_address = env::var("KAFKA_BROKER")
        .expect("Failed to read KAFKA_BROKER from .env");

    let kafka = KafkaProducer::new(&broker_address);

    let db_pool = PgPool::connect(&db_url).await.unwrap();

    let redis = redis::Client::open(redis_url)?;

    let app_state = AppState::new(db_pool, redis, jwt_secret, kafka);

    let connections = app_state.active_connections.clone();

    task::spawn(async move {
        consumer::booking_cancelled_consumer(connections).await;
    });

    let app = Router::new()
        
        .route("/update-location", post(update_location_handler))
        .route("/tracking/{order_id}", get(get_tracking_handler))
        .route("/tracking/ws/{order_id}", get(ws_tracking_handler))
        .route("/tracking/distance", get(calculate_distance_handler))
        .route("/tracking/arrival/{order_id}", post(notify_arrival_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(app_state.clone())
        .layer(from_fn_with_state(
            app_state.clone(), 
            rate_limit_middleware,
        ));

    let addr = "0.0.0.0:3002";
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}