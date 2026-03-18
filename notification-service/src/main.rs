use axum::{
    Router, 
    routing::get,
    middleware::from_fn_with_state
};
use dotenvy::dotenv;
use std::env;
use sqlx::PgPool;
use tokio::task;

mod state;
mod handler;
mod services;
mod util;
mod consumer;
mod errors;
mod middleware;

use state::AppState;
use crate::handler::ws_notifications;
use crate::consumer::consumer;
use crate::middleware::rate_limiting::rate_limit_middleware;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    tracing_subscriber::fmt::init();
    
    let db_url = env::var("DATABASE_URL")
        .expect("Can't find DATABASE_URL in .env");

    let project_id = env::var("FCM_PROJECT_ID")
        .expect("Can't find FCM_PROJECT_ID in .env");

    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set");

    let redis = redis::Client::open(redis_url)?;

    let pool = PgPool::connect(&db_url).await?;

    let state = AppState::new(pool, redis);

    // spawn Kafka consumer
    let state_clone = state.clone();
    task::spawn(async move {
        consumer(state_clone, &project_id
        ).await;
    });

    let app = Router::new()
        .route("/ws/notifications", get(ws_notifications))
        .with_state(state.clone())
        .layer(from_fn_with_state(
            state.clone(), 
            rate_limit_middleware,
        ));

    let addr = "0.0.0.0:3003";
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
