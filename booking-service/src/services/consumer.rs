use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use futures::StreamExt;
use shared_auth::models::{AssignedDetailerEvent, DetailerNotFoundEvent, DetailerArrivedEvent};
use std::sync::Arc;
use sqlx::PgPool;
use std::env;
use crate::services::booking::BookingService;
 
pub async fn consumer(pool: PgPool) {

    let booking_service = Arc::new(
        BookingService::new(pool).await.unwrap()
    );

    let broker = env::var("KAFKA_BROKER")
        .expect("Failed to find consumer in .env");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "booking-service")
        .set("bootstrap.servers", &broker)
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer Failed");

    consumer.subscribe(&["detailer.assigned", "detailer.notfound", "detailer.arrived"]).unwrap();

    let mut stream = consumer.stream();

    tracing::info!("Consumer Started listening to topics: 
        detailer.assigned, detailer.notfound, detailer.arrived");

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

                let topic = msg.topic();

                match  topic {
                    "detailer.assigned" => {

                        let event: AssignedDetailerEvent = match serde_json::from_slice(payload) {
                            Ok(event) => event,
                            Err(e) => {
                                tracing::error!("Failed to Deserialize BookingCreatedEvent: {}", e);
                                continue;
                            }
                        };

                        match booking_service.assign_detailer_in_db(event.order_id, event.detailer_id).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Order: {} is assigned detailer: {}",
                                    event.order_id, 
                                    event.detailer_id
                                );
                            },
                            Err(e) => {
                                tracing::error!("Failed to Update db: {}", e);
                            }
                        }

                    },

                    "detailer.notfound" => {
                        let event: DetailerNotFoundEvent = match serde_json::from_slice(payload) {
                            Ok(event) => event,
                            Err(e) => {
                                tracing::error!("Failed to Deserialize BookingCreatedEvent: {}", e);
                                continue;
                            }
                        };
                    
                        match booking_service.cancel_booking(event.order_id, event.customer_id, None).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Order: {} is cancelled: No detailer found.",
                                    event.order_id, 
                                );
                            },
                            Err(e) => {
                                tracing::error!("Failed to Update db: {}", e);
                            }
                        }
                    }    

                    "detailer.arrived" => {
                        let event: DetailerArrivedEvent = match serde_json::from_slice(payload) {
                            Ok(event) => event,
                            Err(e) => {
                                tracing::error!("Failed to Deserialize BookingCreatedEvent: {}", e);
                                continue;
                            }
                        };

                        match booking_service.order_in_progress_db(event.order_id, event.detailer_id).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Order: {} is in progress.",
                                    event.order_id, 
                                );
                            },
                            Err(e) => {
                                tracing::error!("Failed to Update db: {}", e);
                            }
                        }
                    },

                    _ => {}       
                }

            }
        }

    }
    
}
