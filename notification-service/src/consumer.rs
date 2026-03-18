use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use futures::StreamExt;
use std::env;
use crate::state::AppState;
use crate::services::NotificationService;
use shared_auth::models::{BookingCancelledEvent,  
    DetailerArrivedEvent, 
    DetailerNotFoundEvent, 
    AssignedDetailerEvent
};

pub async fn consumer(state: AppState, project_id: &str) {
    let broker = env::var("KAFKA_BROKER")
        .expect("Failed to find consumer in .env");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "notification-service")
        .set("bootstrap.servers", &broker)
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer Failed");

    consumer.subscribe(&[ 
        "booking.cancelled", 
        "detailer.assigned", 
        "detailer.notfound", 
        "detailer.arrived"
        ])
        .expect("Failed to subscribe");

    tracing::info!("Consumer listening to: booking.created, booking.cancelled, detailer.assigned, detailer.notfound, detailer.arrived");

    let mut stream = consumer.stream();

    while let Some(message) = stream.next().await {
        match message {
            Err(e) => {
                tracing::error!("Kafka Error {}",e);
                continue;
            },

            Ok(msg) => {
                let payload = match msg.payload(){
                    Some(p) => p,
                    None => {
                        tracing::warn!("Empty Payload Received");
                        continue;
                    }
                };

                let topic = msg.topic();

                match topic {
                    
                    "booking.cancelled" => {
                        let event: BookingCancelledEvent = 
                            match serde_json::from_slice(payload) {
                                Ok(event) => event,
                                Err(e) => {
                                    tracing::error!("Failed to parse booking.cancelled event: {}", e);
                                    continue;
                                }
                            };

                        let title = "Booking Cancelled";
                        let body = "Your booking has been cancelled";

                        let record_option = sqlx::query!(
                            "SELECT customer_id FROM orders WHERE id = $1",
                            event.order_id
                        )
                        .fetch_optional(&state.pool)
                        .await
                        .unwrap();

                        if let Some(record) = record_option {

                            let user_id = record.customer_id;

                            // save notifiaction 
                            let _ = NotificationService::save_notification(
                                &state.pool, 
                                user_id,
                                title, 
                                body
                            ).await;
                            
                            // send ws notification
                            let _ = NotificationService::send_ws_notification(
                                &state.ws_connection, 
                                user_id, 
                                title, 
                                body
                            ).await;
                        
                            // send push notification
                            let _ = NotificationService::send_push(
                                user_id, 
                                &state.pool, 
                                project_id, 
                                title, 
                                body
                            ).await;
                        
                            tracing::info!("{}: {}", title, body);
                            
                        }
                    },
                    "detailer.assigned" => {
                        let event: AssignedDetailerEvent = 
                            match serde_json::from_slice(payload) {
                                Ok(event) => event,
                                Err(e) => {
                                    tracing::error!("Failed to parse booking.cancelled event: {}", e);
                                    continue;
                                }
                            };

                        let title = "Detailer Assigned";
                        let body = "You have been assigned a detailer";

                        let record_option = sqlx::query!(
                            "SELECT customer_id FROM orders WHERE id = $1",
                            event.order_id
                        )
                        .fetch_optional(&state.pool)
                        .await
                        .unwrap();

                        if let Some(record) = record_option {

                            let user_id = record.customer_id;

                            // save notifiaction 
                            let _ = NotificationService::save_notification(
                                &state.pool, 
                                user_id,
                                title, 
                                body
                            ).await;
                            
                            // send ws notification
                            let _ = NotificationService::send_ws_notification(
                                &state.ws_connection, 
                                user_id, 
                                title, 
                                body
                            ).await;
                        
                            // send push notification
                            let _ = NotificationService::send_push(
                                user_id, 
                                &state.pool, 
                                project_id, 
                                title, 
                                body
                            ).await;
                        
                            tracing::info!("{}: {}", title, body);
                            
                        }
                    },
                    "detailer.notfound" => {
                        let event: DetailerNotFoundEvent = 
                            match serde_json::from_slice(payload) {
                                Ok(event) => event,
                                Err(e) => {
                                    tracing::error!("Failed to parse booking.cancelled event: {}", e);
                                    continue;
                                }
                            };

                        let title = "Booking Cancelled: Detailer Not Found";
                        let body = "Your booking has been cancelled because no detailer was found.";

                        let user_id = event.customer_id;

                        // save notifiaction 
                        let _ = NotificationService::save_notification(
                            &state.pool, 
                            user_id,
                            title, 
                            body
                        ).await;
                        
                        // send ws notification
                        let _ = NotificationService::send_ws_notification(
                            &state.ws_connection, 
                            user_id, 
                            title, 
                            body
                        ).await;
                    
                        // send push notification
                        let _ = NotificationService::send_push(
                            user_id, 
                            &state.pool, 
                            project_id, 
                            title, 
                            body
                        ).await;
                    
                        tracing::info!("{}: {}", title, body);
                    },

                    "detailer.arrived" => {
                        let event: DetailerArrivedEvent = 
                            match serde_json::from_slice(payload) {
                                Ok(event) => event,
                                Err(e) => {
                                    tracing::error!("Failed to parse booking.cancelled event: {}", e);
                                    continue;
                                }
                            };

                        let title = "Detailer Arrived";
                        let body = "Detailer has arrived to your location";

                        let record_option = sqlx::query!(
                            "SELECT customer_id FROM orders WHERE id = $1",
                            event.order_id
                        )
                        .fetch_optional(&state.pool)
                        .await
                        .unwrap();

                        if let Some(record) = record_option {

                            let user_id = record.customer_id;

                            // save notifiaction 
                            let _ = NotificationService::save_notification(
                                &state.pool, 
                                user_id,
                                title, 
                                body
                            ).await;
                            
                            // send ws notification
                            let _ = NotificationService::send_ws_notification(
                                &state.ws_connection, 
                                user_id, 
                                title, 
                                body
                            ).await;
                        
                            // send push notification
                            let _ = NotificationService::send_push(
                                user_id, 
                                &state.pool, 
                                project_id, 
                                title, 
                                body
                            ).await;
                        
                            tracing::info!("{}: {}", title, body);
                            
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
