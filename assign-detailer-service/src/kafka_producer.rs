use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use shared_auth::models::{AssignedDetailerEvent, DetailerNotFoundEvent};
use crate::errors::{Result, AppError};

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

    pub async fn send_assigned_detailer(&self, event: AssignedDetailerEvent) -> Result<()> {
        let payload = serde_json::to_string(&event)
            .map_err(|_| AppError::InternalServerError("Failed to serialize event".into()))?;

        self.producer.send(
            FutureRecord::to("detailer.assigned")
                .payload(&payload)
                .key(&event.detailer_id.to_string()),
                Duration::from_secs(0),     
        ).await
        .map_err(|_| AppError::InternalServerError("Failed to send Kafka message".into()))?;
        
        Ok(())
    }

    pub async fn detailer_not_found(&self, event: DetailerNotFoundEvent) -> Result<()> {
        let payload = serde_json::to_string(&event)
            .map_err(|_| AppError::InternalServerError("Failed to serialize event".into()))?;

        self.producer.send(
            FutureRecord::to("detailer.notfound")
                .payload(&payload)
                .key(&event.order_id.to_string()),
                Duration::from_secs(0),     
        ).await
        .map_err(|_| AppError::InternalServerError("Failed to send Kafka message".into()))?;
        
        Ok(())
    }
}   