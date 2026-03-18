mod consumer;
mod mailer;

use tracing_subscriber;
use dotenvy::dotenv;
use crate::consumer::start_consumer;

#[tokio::main]
async fn main(){
    tracing_subscriber::fmt::init();

    dotenv().ok();
    
    start_consumer().await;
}