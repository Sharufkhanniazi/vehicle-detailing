use sqlx::PgPool;
use uuid::Uuid;
use std::env;
// Without use std::str::FromStr;, Rust doesn’t know that BigDecimal can be constructed from a string.
use std::str::FromStr;
use bigdecimal::BigDecimal;
use tokio::sync::Mutex;
use tonic::{Request, transport::Channel};
use shared_auth::models::{BookingCreatedEvent, UserRole, BookingCancelledEvent};
use crate::proto::pricing::pricing_service_client::PricingServiceClient;
use crate::services::kafka_producer::KafkaProducer;
use crate::utils::models::{BookingRequest, EstimatePriceRequest, DbVehicleCategory, DbServiceType, OrderStatus};
use crate::utils::error::{AppError, Result};
use crate::proto::pricing::{
    PriceEstimateRequest,
    PriceEstimateResponse,
    VehicleCategory,
};

pub struct BookingService {
    pool: PgPool,
    pricing_client: Mutex<PricingServiceClient<Channel>>,
    kafka: KafkaProducer
}

impl BookingService {
   
    pub async fn new(pool: PgPool) -> Result<Self> {

        let kafka_broker =
            env::var("KAFKA_BROKER")
                .expect("Failed to find KAFKA_BROKER in .env");

        
        let pricing_client_connection =
            PricingServiceClient::connect("http://[::1]:50051")
                .await
                .map_err(|_| AppError::InternalServerError("Can't connect to Pricing-Service".into()))?;

        let pricing_client = Mutex::new(pricing_client_connection);

        let kafka = KafkaProducer::new(&kafka_broker);

        Ok(BookingService {
            pool,
            pricing_client,
            kafka,
        })
    }
    
    pub async fn get_price(
        &self,
        request: EstimatePriceRequest
    ) -> Result<PriceEstimateResponse> {
        let service_enum_i32: Vec<i32> = request
            .services
            .into_iter()
            .map(|s| s as i32)
            .collect();

        
        let pricing_request = PriceEstimateRequest {
            // Because Protobuf enums are integers.
            vehicle: request.vehicle as i32,
            services: service_enum_i32,
        };

        let mut client =self.pricing_client.lock().await;

        let response: PriceEstimateResponse = client
            .get_estimate(Request::new(pricing_request))
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .into_inner();

        Ok(response)
    }

    pub async fn create_booking(
        &self, 
        customer_id: Uuid,
        request: BookingRequest
    ) -> Result<Uuid> {

        let pricing_request = PriceEstimateRequest {
            vehicle: request.vehicle as i32,
            services: request.services.iter().map(|s| *s as i32).collect(),
        };

        let mut client =self.pricing_client.lock().await;

        let estimate = client
            .get_estimate(Request::new(pricing_request))
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .into_inner();

        let subtotal = BigDecimal::from_str(&estimate.subtotal.to_string())?;
        let tax = BigDecimal::from_str(&estimate.tax.to_string())?;
        let surge = BigDecimal::from_str(&estimate.surge_multiplier.to_string())?;
        let total = BigDecimal::from_str(&estimate.total.to_string())?;

        let mut tx = self.pool.begin().await?;

        
        let db_vehicle = Self::proto_vehicle_to_db(request.vehicle)?;

        let order_id: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO orders (
                customer_id,
                brand,
                model,
                vehicle,
                time_slot,
                subtotal,
                tax,
                surge_multiplier,
                total_price,
                latitude,
                longitude
            )
            VALUES (
                $1,$2,$3,$4::vehicle_category,$5,$6,$7,$8,$9,$10,$11
            )
            RETURNING id
            "#,
            customer_id,
            request.brand,
            request.model,
            db_vehicle as DbVehicleCategory,
            request.time_slot,
            subtotal,
            tax,
            surge,
            total,
            request.latitude,
            request.longitude
        )
        .fetch_one(&mut *tx)
        .await?;


        let mut services = Vec::new();

        for service in estimate.services {
            
            let db_service = Self::service_name_to_db(&service.service_name)?;

            let base_price = BigDecimal::from_str(&service.base_price.to_string())?;
            let final_price = BigDecimal::from_str(&service.final_price.to_string())?;

            sqlx::query!(
                r#"
                INSERT INTO order_services (
                    order_id,
                    service,
                    base_price,
                    final_price
                )
                VALUES ($1,$2::service_type,$3,$4)
                "#,
                order_id,
                db_service as DbServiceType,
                base_price,
                final_price
            )
            .execute(&mut *tx)
            .await?;

            services.push(db_service);
        }

        tx.commit().await?;

        tracing::info!("Booking created for {}", customer_id);

        let event = BookingCreatedEvent {
            order_id,
            customer_id,
            latitude: request.latitude, 
            longitude: request.longitude,
            time_slot: request.time_slot
        };

        let _ = self.kafka.send_booking_created(event).await;

        Ok(order_id)
    }

    pub async fn order_completed(
        &self,
        detailer_id: Uuid,
        customer_id: Uuid,
        order_id: Uuid
    ) -> Result<()> {

        let mut tx = self.pool.begin().await?;

        // update order
        let result = sqlx::query!(
            r#"
            UPDATE orders
            SET 
                status = 'COMPLETED',
                updated_at = NOW()
            WHERE id = $1 AND status = 'IN_PROGRESS'
            "#,
            order_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::InternalServerError("Order cannot be completed".into()));
        }

        // update detailer
        sqlx::query!(
            r#"
            UPDATE detailer_profiles
            SET 
                availability_status = 'ONLINE',
                total_jobs_completed = total_jobs_completed + 1,
                updated_at = NOW()
            WHERE user_id = $1
            "#,
            detailer_id
        )
        .execute(&mut *tx)
        .await?;

        // update customer
        sqlx::query!(
            r#"
            UPDATE customer_profiles
            SET
                loyalty_points = loyalty_points + 1,
                updated_at = NOW()
            WHERE user_id = $1
            "#,
            customer_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        
        tracing::info!("order {} completed", order_id);

        Ok(())
    }

    fn proto_vehicle_to_db(vehicle: VehicleCategory) -> Result<DbVehicleCategory> {
        match vehicle {
            VehicleCategory::Small => Ok(DbVehicleCategory::Small),
            VehicleCategory::Sedan => Ok(DbVehicleCategory::Sedan),
            VehicleCategory::Suv => Ok(DbVehicleCategory::Suv),
            VehicleCategory::Truck => Ok(DbVehicleCategory::Truck),
        }
    }

    // Helper function to convert service name to DB enum
    fn service_name_to_db(service_name: &str) -> Result<DbServiceType> {
        match service_name {
            "ExteriorWash" => Ok(DbServiceType::ExteriorWash),
            "InteriorClean" => Ok(DbServiceType::InteriorClean),
            "FullDetailing" => Ok(DbServiceType::FullDetailing),
            "EngineBayCleaning" => Ok(DbServiceType::EngineBayCleaning),
            _ => Err(AppError::InternalServerError(format!("Unknown service type: {}", service_name))),
        }
    }

    pub async fn order_in_progress_db(&self, order_id: Uuid, detailer_id: Uuid) -> Result<()> {
        // update the order
        sqlx::query!(
            r#"
            UPDATE orders
            SET 
                detailer_id = $1,
                status = 'IN_PROGRESS',
                updated_at = NOW()
            WHERE id = $2 AND status = 'ASSIGNED'
            "#,
            detailer_id,
            order_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Order status updated to 'IN_PROGRESS'");

        Ok(())
    }

    pub async fn assign_detailer_in_db(&self, order_id: Uuid, detailer_id: Uuid) -> Result<()>{
        // update the order with assigned detailer
        sqlx::query!(
            r#"
            UPDATE orders
            SET 
                detailer_id = $1,
                status = 'ASSIGNED',
                updated_at = NOW()
            WHERE id = $2 AND status = 'PENDING'
            "#,
            detailer_id,
            order_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE detailer_profiles
            SET 
                availability_status = 'BUSY',
                updated_at = NOW()
            WHERE user_id = $1
            "#,
            detailer_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Detailer added in db");

        Ok(())
    }

    pub async fn cancel_booking(
        &self, 
        order_id: Uuid, 
        user_id: Uuid,
        user_role: Option<UserRole>,
    ) -> Result<()> {

        let order = sqlx::query!(
            r#"
            SELECT detailer_id, status as "order_status: OrderStatus"
            FROM orders
            WHERE id = $1
            AND (customer_id = $2 OR detailer_id = $2)
            "#,
            order_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let order = match order {
            Some(o) => o,
            None => return Err(AppError::InternalServerError("Order not found or not authorized".into())),
        };

        // Check if order can be cancelled (optional - based on your business logic)
        if order.order_status == OrderStatus::COMPLETED || order.order_status == OrderStatus::CANCELLED {
            return Err(AppError::InternalServerError(format!("Order cannot be cancelled as it is {:?}", order.order_status)));
        }
        
        let result = sqlx::query!(
            r#"
            UPDATE orders
            SET 
                status = 'CANCELLED',
                updated_at = NOW()
            WHERE id = $1
            AND (customer_id = $2 OR detailer_id = $2)
            "#,
            order_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::InternalServerError("Failed to cancel order".into()));
        }

        if let Some(detailer_id) = order.detailer_id {
            sqlx::query!(
                r#"
                UPDATE detailer_profiles
                SET 
                    availability_status = 'ONLINE',
                    updated_at = NOW()
                WHERE user_id = $1
                "#,
                detailer_id
            )
            .execute(&self.pool)
            .await?;
        }

        tracing::info!("Order {} cancelled", order_id);

        if let Some(role) = user_role {
            let event = BookingCancelledEvent {
                order_id,
                user_id,
                role
            };

            let _ = self.kafka.send_booking_cancelled(event).await?;
        }
        
        Ok(())
    }

    pub async fn submit_review(
        &self,
        order_id: Uuid,
        customer_id: Uuid,
        detailer_id: Uuid,
        rating: i32, // 1-5
        comment: Option<String>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Validate the order
        let order = sqlx::query!(
            r#"
            SELECT status as "status!: OrderStatus", customer_id, detailer_id
            FROM orders
            WHERE id = $1
            "#,
            order_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if order.status != OrderStatus::COMPLETED {
            return Err(AppError::InternalServerError("Order isn't COMPLETED".into())); // Or define a custom error
        }

        if order.customer_id != customer_id || order.detailer_id != Some(detailer_id) {
            return Err(AppError::InternalServerError("Order doesn't belong to customer or detailer".into())); // Invalid customer or detailer
        }

        // Insert the review
        sqlx::query!(
            r#"
            INSERT INTO reviews (order_id, customer_id, detailer_id, rating, comment)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            order_id,
            customer_id,
            detailer_id,
            rating,
            comment
        )
        .execute(&mut *tx)
        .await?;

        // Update running totals & average
        sqlx::query!(
            r#"
            UPDATE detailer_profiles
            SET
                total_rating_points = total_rating_points + $1,
                total_reviews = total_reviews + 1,
                rating = ROUND(
                    (total_rating_points + $1)::numeric /
                    (total_reviews + 1),
                    2
                )
            WHERE user_id = $2
            "#,
            rating,
            detailer_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}