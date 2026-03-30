mod common;

use uuid::Uuid;
use chrono::{Utc, Duration};
use sqlx::query;

#[tokio::test]
async fn test_assign_nearest_detailer() {
    let (service, test_db) = common::create_test_service().await;
    
    // insert a detailer
    let detailer_id = Uuid::new_v4();
    
    query!(
        r#"
        INSERT INTO users (id, role, is_active)
        VALUES ($1, 'DETAILER', true)
        "#,
        detailer_id
    ).execute(&test_db.pool).await.unwrap();
    
    query!(
        r#"
        INSERT INTO detailer_profiles (user_id, last_known_latitude, last_known_longitude, availability_status)
        VALUES ($1, 40.7128, -74.0060, 'ONLINE')
        "#,
        detailer_id
    ).execute(&test_db.pool).await.unwrap();
    
    let time_slot = Utc::now() + Duration::hours(2);
    let result = service.assign_detailer(40.7128, -74.0060, time_slot).await.unwrap();
    
    assert_eq!(result, Some(detailer_id));
    
    test_db.cleanup().await;
}

#[tokio::test] 
async fn no_detailer_assigned() { 
    // no detailer with-in 10 km Range
    let (service, test_db) = common::create_test_service().await;
    
    // insert a detailer
    let detailer_id = Uuid::new_v4();
    
    query!(
        r#"
        INSERT INTO users (id, role, is_active)
        VALUES ($1, 'DETAILER', true)
        "#,
        detailer_id
    ).execute(&test_db.pool).await.unwrap();
    
    query!(
        r#"
        INSERT INTO detailer_profiles (user_id, last_known_latitude, last_known_longitude, availability_status)
        VALUES ($1, 40.7128, -74.0060, 'ONLINE')
        "#,
        detailer_id
    ).execute(&test_db.pool).await.unwrap();
    
    let time_slot = Utc::now() + Duration::hours(2);
    let result = service.assign_detailer(0.000,     0.000, time_slot).await.unwrap();
    
    assert_eq!(result, None);
    
    test_db.cleanup().await;
}