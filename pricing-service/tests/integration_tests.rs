use pricing_service::pricing::{
    pricing_service_client::PricingServiceClient,
    PriceEstimateRequest, VehicleCategory, ServiceType
};

use pricing_service::pricing::pricing_service_server::PricingServiceServer;
use pricing_service::MyPricingService;

use tonic::transport::Server;
use tokio::time::{sleep, Duration};

async fn start_test_server(addr: &str) {
    let addr = format!("[::1]:{}", addr).parse().unwrap();

    tokio::spawn(async move {
        Server::builder()
            .add_service(PricingServiceServer::new(MyPricingService))
            .serve(addr)
            .await
            .unwrap(); 
    });

    // pause test for 300 milli seconds
    // giving gRPC server time to fully start
    sleep(Duration::from_millis(300)).await; 
}

#[tokio::test]
async fn test_grpc_price_estimate() {
    start_test_server("50052").await;

    let mut client = 
        PricingServiceClient::connect("http://[::1]:50052")
        .await
        .unwrap();

    let request = PriceEstimateRequest {
        vehicle: VehicleCategory::Sedan as i32,
        services: vec![ServiceType::ExteriorWash as i32]
    };

    let response = client.get_estimate(request).await.unwrap().into_inner();

    assert_eq!(response.currency, "PKR");

    assert_eq!(response.services.len(), 1);
    
    assert_eq!(response.subtotal, 1800.0);
}

#[tokio::test]
async fn test_grpc_invalid_request() {
    start_test_server("50053").await;

    let mut client = 
        PricingServiceClient::connect("http://[::1]:50053")
        .await
        .unwrap();

    let request = PriceEstimateRequest {
        vehicle: 10,
        services: vec![10]
    };

    let result = client.get_estimate(request).await;

    assert!(result.is_err())
}