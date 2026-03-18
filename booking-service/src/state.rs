use std::sync::Arc;
use crate::services::booking::BookingService;

#[derive(Clone)]
pub struct AppState {
    pub booking_service: Arc<BookingService>,
    pub redis: redis::Client, // rate limiting
    pub jwt_secret: String
}

impl AppState {
    pub fn new(jwt_secret: String, booking_service: Arc<BookingService>, redis: redis::Client) -> Self {
        
        AppState { booking_service, redis, jwt_secret }
    }
}