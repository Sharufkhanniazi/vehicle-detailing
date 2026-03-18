use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use futures::StreamExt;
use shared_auth::models::UserCreatedEvent;
use std::env;
use crate::mailer::send_email_verification;

pub async fn start_consumer() {

    let broker = env::var("KAFKA_BROKER")
        .expect("Failed to find consumer in .env");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "email-service")
        .set("bootstrap.servers", &broker)
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer Failed");

    consumer.subscribe(&["user.created"]).unwrap();

    // Converts kafka consumer into an async stream
    // now instead of manually polling now you:
    // Await messages
    // Process them as they arrive
    let mut stream = consumer.stream();

    tracing::info!("Consumer Started listening to topic: user.created");

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

                let event: UserCreatedEvent = match serde_json::from_slice(payload) {
                    Ok(event) => event,
                    Err(e) => {
                        tracing::error!("Failed to Deserialize UserCreatedEvent: {}", e);
                        continue;
                    }
                };

                match send_email_verification(&event.email, &event.email_token).await {
                    Ok(_) => { tracing::info!("Verification email send successfully") },
                    Err(e) => { tracing::error!("Failed to send verification email: {}",e)}
                }
            }
        }
    }
    
}