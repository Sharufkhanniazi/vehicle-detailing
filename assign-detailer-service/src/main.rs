mod consumer;
mod assign_detailer;
mod errors;
mod kafka_producer;

use dotenvy::dotenv;
use crate::consumer::start_consumer;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt::init();
    
    start_consumer().await;
}
