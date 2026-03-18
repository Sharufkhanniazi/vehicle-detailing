use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use shared_auth::models::DetailerArrivedEvent;

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    // kafka producer constructor
    pub fn new(brokers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()// bootstrap.servers is the only required configuration for the producer,
            .set("bootstrap.servers", brokers) // Tells Kafka where the brokers are.
            .create()
            .expect("Failed to create Kafka producer");

        Self { producer }
    }

    pub async fn detailer_arrived(&self, event: DetailerArrivedEvent) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_string(&event).unwrap();

        self.producer.send(
            FutureRecord::to("detailer.arrived")
                .payload(&payload)
                .key(&event.order_id.to_string()),
                Duration::from_secs(0),     
        ).await
        .unwrap();

        tracing::info!("DetailerArrivedEvent send to detailer.arrived");
        
        Ok(())
    }
}   