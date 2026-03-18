use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::mpsc;
use axum::extract::ws::Message;
use crate::services::kafka_producer::KafkaProducer;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool, 
    pub redis: redis::Client,
    pub active_connections: Arc<DashMap<Uuid, Vec<mpsc::UnboundedSender<Message>>>>,
    pub kafka: KafkaProducer,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(pool: PgPool, redis: redis::Client, jwt_secret: String, kafka: KafkaProducer ) -> Self {
        let active_connections =
            Arc::new(DashMap::<Uuid, Vec<mpsc::UnboundedSender<Message>>>::new());

        AppState { pool, redis, active_connections, jwt_secret, kafka}
    }
}