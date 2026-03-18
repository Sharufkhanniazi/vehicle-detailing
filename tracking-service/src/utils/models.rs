use serde::{Deserialize, Serialize};
use shared_auth::models::UserRole;
use uuid::Uuid;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Deserialize)]
pub struct LocationUpdate {
    pub order_id: Option<Uuid>,
    pub latitude: f64,
    pub longitude: f64, 
    pub timestamp: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct DistanceQuery {
    pub lat1: f64,
    pub lng1: f64,
    pub lat2: f64,
    pub lng2: f64,
}

#[derive(Debug, Deserialize)]
pub struct CachedLocation {
    pub lat: f64,
    pub lng: f64,
    pub timestamp: DateTime<Utc>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackingInfo {
    pub order_id: Uuid,
    pub detailer_id: Uuid,
    pub detailer_lat: f64,
    pub detailer_lng: f64,
    pub customer_lat: f64,
    pub customer_lng: f64,
    pub distance_km: f64,
    pub eta_minutes: i32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    LocationUpdate {
        order_id: Uuid,
        latitude: f64,
        longitude: f64,
        distance_km: f64,
        eta_minutes: i32,
        timestamp: DateTime<Utc>
    },
    StatusUpdate {
        order_id: Uuid,
        status: String,
        cancelled_by: Uuid,
        role: UserRole
    },
    Arrived {
        order_id: Uuid,
        message: String
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DistanceResponse {
    pub distance_km: f64,
    pub eta_minutes: i32,
}