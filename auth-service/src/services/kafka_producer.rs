use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use crate::utils::error::{Result, AppError};
use shared_auth::models::UserCreatedEvent;

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    // kafka producer constructor
    pub fn new(brokers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers) 
            .create()
            .expect("Failed to create Kafka producer");

        Self { producer }
    }

    pub async fn send_user_created(&self, event: UserCreatedEvent) -> Result<()> {
        let payload = serde_json::to_string(&event)
            .map_err(|_| AppError::InternalServerError("Failed to serialize event".into()))?;

        self.producer.send(
            FutureRecord::to("user.created")
                .payload(&payload)
                .key(&event.user_id.to_string()),
                Duration::from_secs(0), 
        ).await
        .map_err(|_| AppError::InternalServerError("Failed to send Kafka message".into()))?;
        
        Ok(())
    }
}