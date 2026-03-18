use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use futures::StreamExt;
use shared_auth::models::BookingCreatedEvent;
use shared_auth::models::{AssignedDetailerEvent, DetailerNotFoundEvent};
use std::sync::Arc;
use std::env;
use crate::assign_detailer::DetailerAssignmentService;

pub async fn start_consumer() {

    // This avoids: recreating DB pools, recreating Kafka producers
    let detailer_assignment_service = Arc::new(
        DetailerAssignmentService::new()
        .await
        .expect("Failed to initialize AssignDetailer")
    );

    let broker = env::var("KAFKA_BROKER")
        .expect("Failed to find consumer in .env");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "assign-detailer-service")
        .set("bootstrap.servers",&broker)
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer Failed");

    consumer.subscribe(&["booking.created"]).unwrap();

    // Converts kafka consumer into an async stream
    // now instead of manually polling now you:
    // Await messages
    // Process them as they arrive
    let mut stream = consumer.stream();

    tracing::info!("Consumer Started listening to topic: booking.created");

    // future improvement: tokio::spawn(async move { ... })
    // Right now your consumer processes Kafka messages sequentially, 
    // Using tokio::spawn allows parallel processing.
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

                let event: BookingCreatedEvent = match serde_json::from_slice(payload) {
                    Ok(event) => event,
                    Err(e) => {
                        tracing::error!("Failed to Deserialize BookingCreatedEvent: {}", e);
                        continue;
                    }
                };

                match detailer_assignment_service.assign_detailer(event.latitude, event.longitude, event.time_slot).await {
                    Ok(detailer_id) => {
                        match detailer_id {
                            Some(detailer_id) => { 
                                tracing::info!(
                                    order_id = %event.order_id,
                                    detailer_id = %detailer_id,
                                    "Detailer assigned successfully"
                                );

                                let event = AssignedDetailerEvent { 
                                    detailer_id , 
                                    order_id: event.order_id 
                                };

                                detailer_assignment_service.kafka
                                .send_assigned_detailer(event)
                                .await
                                .unwrap();
                            },
                            None => { 
                                tracing::error!("No Detailer available.");

                                let event = DetailerNotFoundEvent {
                                    order_id: event.order_id,
                                    customer_id: event.customer_id
                                };

                                detailer_assignment_service.kafka
                                    .detailer_not_found(event)
                                    .await
                                    .unwrap();
                            }
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to assign detailer: {}", e);
                    }
                }
            }
        }
    }
    
}