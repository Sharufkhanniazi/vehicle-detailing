use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use redis::AsyncCommands;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::mpsc;
use axum::extract::ws::{Message, Utf8Bytes};
use crate::utils::models::{LocationUpdate, CachedLocation, TrackingInfo, WsMessage};
use crate::utils::errors::{Result, AppError};

pub struct TrackingService;

impl TrackingService {
    
    pub async fn update_location(
        pool: &PgPool,
        redis: &redis::Client,
        update: LocationUpdate,
        detailer_id: Uuid
    ) -> Result<()> {
        // Update Postgresql
        sqlx::query!(
            r#"
            UPDATE detailer_profiles
            SET 
                last_known_latitude = $1,
                last_known_longitude = $2,
                updated_at = NOW()
            WHERE user_id = $3
            "#,
            update.latitude,
            update.longitude,
            detailer_id,
        )
        .execute(pool)
        .await?;

        let mut redis_conn = redis.get_multiplexed_async_connection()
            .await
            .map_err(|_| AppError::InternalServerError("Redis Error".into()))?;

        let redis_key = format!("location:{}", detailer_id);
        let redis_value = serde_json::json!({
            "lat": update.latitude,
            "lng": update.longitude,
            "timestamp": update.timestamp
        });

        let _: () = redis_conn.set_ex(
            &redis_key, 
            redis_value.to_string(), 
            300
        )
        .await
        .map_err(|_| AppError::InternalServerError("Redis Error".into()))?;

        tracing::info!("Detailer {} location updated", detailer_id);

        Ok(())
    }
    
    pub async fn get_detailer_location(
        pool: &PgPool,
        redis: &redis::Client,
        detailer_id: Uuid
    ) -> Result<(f64, f64, DateTime<Utc>)> {

        let mut redis_conn = redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| AppError::InternalServerError("Redis Error".into()))?;
        
        let redis_key = format!("location:{}",detailer_id);

        if let Ok(Some(cached)) = redis_conn.get::<_, Option<String>>(&redis_key).await {
            if let Ok(loc) = serde_json::from_str::<CachedLocation>(&cached) {
                return Ok((loc.lat, loc.lng, loc.timestamp));
            }
        }

        // fallback to db
        let profile = sqlx::query!(
            r#"
            SELECT last_known_latitude, last_known_longitude, updated_at
            FROM detailer_profiles
            WHERE user_id = $1
            "#,
            detailer_id
        )
        .fetch_one(pool)
        .await?;

        tracing::info!("Detailer {} location received", detailer_id);

        Ok((
            profile.last_known_latitude,
            profile.last_known_longitude,
            profile.updated_at
        ))
    
    }

    // Calculate distance between two points (Haversine formula)
    pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0; // Earth's radius in km
        
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        
        let a = (dlat / 2.0).sin() * (dlat / 2.0).sin() +
                (lat1.to_radians().cos()) * (lat2.to_radians().cos()) *
                (dlon / 2.0).sin() * (dlon / 2.0).sin();
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        r * c
    }

    // Estimated Time of Arrival.
    // Calculate ETA based on distance (simplified - assumes 30 km/h average speed)
    pub fn calculate_eta(distance_km: f64) -> i32 {
        (distance_km / 30.0 * 60.0).round() as i32
    }

    pub async fn get_tracking_info(
        pool: &PgPool,
        redis: &redis::Client,
        order_id: Uuid,
        user_id: Uuid // for authorization 
    ) -> Result<TrackingInfo> {

        let order = sqlx::query!(
            r#"
            SELECT 
                o.id,
                o.customer_id,
                o.detailer_id,
                o.latitude as customer_lat,
                o.longitude as customer_lng
            FROM orders o 
            WHERE o.id = $1
                AND (o.customer_id = $2 OR o.detailer_id = $2)
            "#,
            order_id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        let detailer_id = order.detailer_id
            .ok_or(AppError::InternalServerError("No detailer assined yet".into()))?;

        let (detailer_lat, detailer_lng, last_updated) =
            Self::get_detailer_location(pool, redis, detailer_id).await?;

        let distance_km = Self::calculate_distance(
            detailer_lat, detailer_lng,     
            order.customer_lat, order.customer_lng
        );

        let eta_minutes = Self::calculate_eta(distance_km);

        tracing::info!("Tracking info received");

        Ok(TrackingInfo {
            order_id,
            detailer_id,
            detailer_lat,
            detailer_lng,
            customer_lat: order.customer_lat,
            customer_lng: order.customer_lng,
            distance_km,
            eta_minutes,
            last_updated
        })
    }

    pub async fn broadcast_to_order(
        connections: &Arc<DashMap<Uuid, Vec<mpsc::UnboundedSender<Message>>>>,
        order_id: Uuid,
        message: WsMessage
    ) {
        if let Some(entry) = connections.get(&order_id) {
            let senders = entry.value();
            let message_str = serde_json::to_string(&message).unwrap();

            tracing::info!("Broadcast Message: {}", message_str);
            
            for sender in senders {
                let _ = sender.send(Message::Text(Utf8Bytes::from(message_str.clone())));
            }
        } 
    }
}