use sqlx::PgPool;
use std::{env, sync::Arc};
use dashmap::DashMap;
use tokio::sync::mpsc;
use axum::extract::ws::Message;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ws_connection: Arc<DashMap<Uuid , Vec<mpsc::UnboundedSender<Message>>>>,
    pub redis: redis::Client, // rate limiting
    pub jwt_secret: String
}

impl AppState {
    pub fn new(pool: PgPool, redis: redis::Client) -> Self {
        let jwt_secret = env::var("JWT_SECRET_KEY")
            .expect("Can't find jwt key in .env");

        AppState { 
            pool, 
            ws_connection: Arc::new(DashMap::new()), 
            redis,
            jwt_secret
        }
    }
}