use chrono::{Duration, Utc};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod pricing {
    tonic::include_proto!("pricing");
}

use pricing::pricing_service_server::PricingService;
use pricing::{PriceEstimateRequest, 
    PriceEstimateResponse, 
    ServiceBreakdown, 
    VehicleCategory, 
    ServiceType
};

pub struct MyPricingService;

#[tonic::async_trait]
impl PricingService for MyPricingService {
    async fn get_estimate(
        &self, 
        request: Request<PriceEstimateRequest>
    ) -> Result<Response<PriceEstimateResponse>, Status> {

        let req = request.into_inner();

        // VehicleCategory::from_i32(req.vehicle)
        // re-construct VehicleCategory from i32
        #[allow(deprecated)]
        let vehicle_multiplier = match VehicleCategory::from_i32(req.vehicle) {
            Some(VehicleCategory::Small) => 1.0,
            Some(VehicleCategory::Sedan) => 1.2,
            Some(VehicleCategory::Suv) => 1.5,
            Some(VehicleCategory::Truck) => 1.8,
            None => return Err(Status::invalid_argument("Invalid vehicle category")),
        };
        
        if req.services.is_empty() {
            return Err(Status::invalid_argument("At least one service required"));
        }

        let mut subtotal = 0.0;
        let mut breakdown = Vec::new();

        for service in req.services {
            // ServiceType::from_i32(service)
            // re-construct ServiceType from i32 (service)
            #[allow(deprecated)]
            let service_enum = ServiceType::from_i32(service)
                .ok_or_else(|| Status::invalid_argument("Invalid service type"))?;

            let base_price = match service_enum {
                ServiceType::ExteriorWash => 1500.0,
                ServiceType::InteriorClean => 2000.0,
                ServiceType::FullDetailing => 5000.0,
                ServiceType::EngineBayCleaning => 1800.0,
            };

            let final_price = base_price * vehicle_multiplier;

            subtotal += final_price;

            breakdown.push(ServiceBreakdown {
                service_id: format!("{:?}", service_enum),
                service_name: format!("{:?}", service_enum),
                base_price,
                final_price
            });
        }

        // surge logic
        let surge_multiplier = 1.0;
        let tax = subtotal * 0.10;
        let total = (subtotal + tax) * surge_multiplier;

        let response = PriceEstimateResponse {
            estimate_id: Uuid::new_v4().to_string(),
            currency: "PKR".to_string(),
            subtotal,
            tax,
            surge_multiplier,
            total,
            services: breakdown,
            expires_at: Utc::now()
                .checked_add_signed(Duration::minutes(5))
                .unwrap()
                .to_rfc3339(),
        };

        tracing::info!(
            "Estimate generated for Vehicle service | Total: {}",
            total
        );

        Ok(Response::new(response))
    }
}