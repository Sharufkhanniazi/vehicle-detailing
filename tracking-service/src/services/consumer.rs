use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use futures::StreamExt;
use uuid::Uuid;
use std::sync::Arc;
use std::env;
use dashmap::DashMap;
use tokio::sync::mpsc;
use axum::extract::ws::Message as AxumMessage;
use shared_auth::models::BookingCancelledEvent;

pub async fn booking_cancelled_consumer(
    connections: Arc<DashMap<Uuid, Vec<mpsc::UnboundedSender<AxumMessage>>>>,
) {

    let broker = env::var("KAFKA_BROKER")
        .expect("Failed to find consumer in .env");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "tracking-service")
        .set("bootstrap.servers", &broker)
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer Failed");

    consumer.subscribe(&["booking.cancelled"]).unwrap();

    let mut stream = consumer.stream();

    tracing::info!("Consumer Started listening to topic: booking.cancelled");

    while let Some(message) = stream.next().await {
        match message {
            Err(e) => {
                tracing::error!("Kakfa error occurred while receiving message: {}",e);
                continue;
            } 
            Ok(msg) => {

                let payload = match msg.payload() {
                    Some(p) => p,
                    None => {
                        tracing::warn!("Received message with empty payload");
                        continue;
                    }
                };

                let event: BookingCancelledEvent = match serde_json::from_slice(payload) {
                    Ok(event) => event,
                    Err(e) => {
                        tracing::error!("Failed to Deserialize BookingCreatedEvent: {}", e);
                        continue;
                    }
                };
                
                tracing::info!("Closing ws connections for tracking detailer.");

                connections.remove(&event.order_id);
                    
            }
        }

    }
    
}
