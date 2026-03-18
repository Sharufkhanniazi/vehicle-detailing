use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::Type;
use chrono::{DateTime, Utc};


#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role")] // matches postgres type (enum name)
#[sqlx(rename_all = "UPPERCASE")] // maps Rust CUSTOMER -> PostgreSQL 'CUSTOMER'
pub enum UserRole {
    CUSTOMER, // Now sqlx can send CUSTOMER as 'CUSTOMER' to PostgreSQL
    DETAILER,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub username: String,
    pub role: UserRole,
    pub exp: usize,
}

// Claims for email verification tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationClaims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub email: String,
    pub email_token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookingCreatedEvent {
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub latitude: f64, 
    pub longitude: f64,
    pub time_slot: DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookingCancelledEvent {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub role: UserRole
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignedDetailerEvent {
    pub detailer_id: Uuid,
    pub order_id: Uuid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailerNotFoundEvent {
    pub order_id: Uuid,
    pub customer_id: Uuid
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DetailerArrivedEvent {
    pub order_id: Uuid,
    pub detailer_id: Uuid
}
