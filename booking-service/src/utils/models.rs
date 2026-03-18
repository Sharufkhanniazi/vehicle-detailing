use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::Type;
use uuid::Uuid;
use crate::proto::pricing::{VehicleCategory, ServiceType};

#[derive(Deserialize)]
pub struct EstimatePriceRequest {
    pub vehicle: VehicleCategory,
    pub services: Vec<ServiceType>,
}

#[derive(Deserialize)]
pub struct BookingRequest {
    pub brand: String,
    pub model: String,
    pub vehicle: VehicleCategory,
    pub services: Vec<ServiceType>,
    pub time_slot: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "vehicle_category", rename_all = "UPPERCASE")]
pub enum DbVehicleCategory {
    Small,
    Sedan,
    Suv,
    Truck
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "service_type", rename_all = "PascalCase")]
pub enum DbServiceType {
    ExteriorWash,
    InteriorClean,
    FullDetailing,
    EngineBayCleaning,
}

#[allow(non_camel_case_types)]
#[derive(sqlx::Type, Debug, PartialEq)]
#[sqlx(type_name = "order_status", rename_all = "UPPERCASE")]
pub enum OrderStatus {
    PENDING,
    AWAITING_PAYMENT,
    CONFIRMED,
    ASSIGNED,
    IN_PROGRESS,
    COMPLETED,
    CANCELLED
}

#[derive(Debug, Deserialize)]
pub struct CancelBooking {
    pub order_id: Uuid
}

#[derive(Debug, Deserialize)]
pub struct OrderCompleted {
    pub customer_id: Uuid,
    pub order_id: Uuid
}

#[derive(Deserialize)]
pub struct SubmitReviewRequest {
    pub detailer_id: Uuid,
    pub order_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
}