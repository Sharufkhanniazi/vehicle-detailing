use tonic::transport::Server;

use pricing_service::pricing::pricing_service_server::PricingServiceServer;
use pricing_service::MyPricingService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::fmt::init();
    
    let addr = "[::1]:50051".parse()?;

    let service = MyPricingService;

    tracing::info!("Order service running on 50051");

    Server::builder()
        .add_service(PricingServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
