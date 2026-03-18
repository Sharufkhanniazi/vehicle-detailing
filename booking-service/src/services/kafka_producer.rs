use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use crate::utils::error::{Result, AppError};
use shared_auth::models::{BookingCreatedEvent, BookingCancelledEvent};

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

    pub async fn send_booking_created(&self, event: BookingCreatedEvent) -> Result<()> {
        let payload = serde_json::to_string(&event)
            .map_err(|_| AppError::InternalServerError("Failed to serialize event".into()))?;

        self.producer.send(
            FutureRecord::to("booking.created")
                .payload(&payload)
                .key(&event.order_id.to_string()),
                Duration::from_secs(0),     
        ).await
        .map_err(|_| AppError::InternalServerError("Failed to send Kafka message".into()))?;

        tracing::info!("BookingCreatedEvent send to booking.created");
        
        Ok(())
    }

    pub async fn send_booking_cancelled(&self, event: BookingCancelledEvent) -> Result<()> {
        let payload = serde_json::to_string(&event)
            .map_err(|_| AppError::InternalServerError("Failed to serialize event".into()))?;

        self.producer.send(
            FutureRecord::to("booking.cancelled")
                .payload(&payload)
                .key(&event.order_id.to_string()),
                Duration::from_secs(0),     
        ).await
        .map_err(|_| AppError::InternalServerError("Failed to send Kafka message".into()))?;

        tracing::info!("BookingCancelledEvent send to booking.cancelled");
        
        Ok(())
    }
}   