mod common;

use chrono::Utc;
use tracking_service::services::tracking::TrackingService;
use tracking_service::utils::models::{LocationUpdate, WsMessage};
use common::test_db;
use std::env;
use uuid::Uuid;
use tokio::sync::mpsc;
use dashmap::DashMap;
use std::sync::Arc;

#[tokio::test]
async fn haversine_distance() {
    let distance = TrackingService::calculate_distance(0.0, 0.0, 0.0, 1.0);

    // ~111 km for 1 degree longitude at equator
    // take absolute value (ignoring + / -)
    assert!((distance - 111.0).abs() < 1.0)
}

#[tokio::test]
async fn update_and_get_location_test() -> anyhow::Result<()> {
    let test_db = test_db::TestDb::new().await;
    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set");

    let redis = redis::Client::open(redis_url)?;

    let detailer_id = Uuid::new_v4();

    // insert detailer
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at)
        VALUES ($1, 'testuser', 'detailer@test.com', 'hash', 'DETAILER', NOW(), NOW())
        "#,
        detailer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO detailer_profiles (user_id, availability_status, total_jobs_completed, total_rating_points, total_reviews, rating, created_at, updated_at)
        VALUES ($1, 'ONLINE', 0, 0, 0, 0.0, NOW(), NOW())
        "#,
        detailer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    let update = LocationUpdate { 
        order_id: None, 
        latitude: 1.11, 
        longitude: 1.11, 
        timestamp: Utc::now() 
    };

    TrackingService::update_location(&test_db.pool, &redis, update, detailer_id).await?;

    let (latitude, longitude, _updated_at) = TrackingService::get_detailer_location(&test_db.pool, &redis, detailer_id).await?;

    assert_eq!(latitude, 1.11);
    assert_eq!(longitude, 1.11);

    test_db.cleanup().await;

    Ok(())
}

#[tokio::test]
async fn test_broadcast_to_order() {
    let connections = Arc::new(DashMap::new());

    let order_id = Uuid::new_v4();

    let (tx, mut rx) = mpsc::unbounded_channel();

    connections.insert(order_id, vec![tx.clone()]);

    let message = WsMessage::LocationUpdate {
        order_id,
        latitude: 1.0,
        longitude: 2.0,
        distance_km: 10.0,
        eta_minutes: 20,
        timestamp: Utc::now(),
    };

    TrackingService::broadcast_to_order(&connections, order_id, message).await;

    let received = rx.recv().await.unwrap();

    match received {
        axum::extract::ws::Message::Text(_) => assert!(true),
        _ => panic!("Expected text message"),
    }
}