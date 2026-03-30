mod common;

// import service modules
use booking_service::{
    services::booking::BookingService,
    utils::models::{
        BookingRequest, DbVehicleCategory, OrderStatus
    },
    proto::pricing::{VehicleCategory, ServiceType},
};

use std::sync::Arc;
use chrono::{Duration, Utc};
use uuid::Uuid;
use sqlx::Type;
use crate::common::test_db::TestDb;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "detailer_availability", rename_all = "UPPERCASE")]
pub enum DetailerAvailability {
    ONLINE, 
    OFFLINE, 
    BUSY  
}

#[tokio::test]
async fn test_create_and_get_booking() {
    let test_db = TestDb::new().await;

    // insert test data
    let customer_id = Uuid::new_v4();
    let detailer_id = Uuid::new_v4();

    // insert customer
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
        VALUES ($1, 'customer@test.com', 'hash', 'CUSTOMER', NOW(), NOW())
        "#,
        customer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO customer_profiles (user_id, loyalty_points, created_at, updated_at)
        VALUES ($1, 0, NOW(), NOW())
        "#,
        customer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    // insert detailer
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
        VALUES ($1, 'detailer@test.com', 'hash', 'DETAILER', NOW(), NOW())
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

    // create BookingService
    let booking_service = Arc::new(
        BookingService::new(test_db.pool.clone())
            .await
            .unwrap()
    );

    // create booking request
    let request = BookingRequest {
        brand: "Toyota".to_string(),
        model: "Camry".to_string(),
        vehicle: VehicleCategory::Sedan,
        services: vec![ServiceType::ExteriorWash, ServiceType::InteriorClean],
        time_slot: Utc::now() + Duration::hours(24),
        latitude: 40.7128,
        longitude: -74.0060,
    };

    // create booking
    let order_id = booking_service
        .create_booking(customer_id, request)
        .await
        .unwrap();

    // verify order was created
    let order = sqlx::query!(
        r#"
        SELECT brand, model, vehicle as "vehicle: DbVehicleCategory", 
               status as "status: OrderStatus", customer_id
        FROM orders
        WHERE id = $1
        "#,
        order_id
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();

    assert_eq!(order.brand, "Toyota");
    assert_eq!(order.customer_id, customer_id);
    assert_eq!(order.status, OrderStatus::PENDING);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_order_completion_flow() {
    let test_db = TestDb::new().await;

    // insert test data
    let customer_id = Uuid::new_v4();
    let detailer_id = Uuid::new_v4();
    let order_id = Uuid::new_v4();

    // insert users
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
        VALUES 
            ($1, 'customer@test.com', 'hash', 'CUSTOMER', NOW(), NOW()),
            ($2, 'detailer@test.com', 'hash', 'DETAILER', NOW(), NOW())
        "#,
        customer_id,
        detailer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    // insert customer profile
    sqlx::query!(
        r#"
        INSERT INTO customer_profiles (user_id, loyalty_points, created_at, updated_at)
        VALUES ($1, 5, NOW(), NOW())
        "#,
        customer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    // insert detailer profile
    sqlx::query!(
        r#"
        INSERT INTO detailer_profiles (user_id, availability_status, total_jobs_completed, 
                                       total_rating_points, total_reviews, rating, created_at, updated_at)
        VALUES ($1, 'BUSY', 10, 45, 10, 4.5, NOW(), NOW())
        "#,
        detailer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    // insert order
    sqlx::query!(
        r#"
        INSERT INTO orders (id, customer_id, detailer_id, brand, model, vehicle, time_slot, 
                           subtotal, tax, surge_multiplier, total_price, latitude, longitude, 
                           status, created_at, updated_at)
        VALUES ($1, $2, $3, 'Honda', 'Civic', 'SEDAN', NOW(), 
                100.0, 10.0, 1.0, 110.0, 40.71, -74.00, 'IN_PROGRESS', NOW(), NOW())
        "#,
        order_id,
        customer_id,
        detailer_id
    )
    .execute(&test_db.pool)
    .await
    .unwrap();

    // create BookingService
    let booking_service = Arc::new(
        BookingService::new(test_db.pool.clone())
            .await
            .unwrap()
    );

    // complete the order
    booking_service
        .order_completed(detailer_id, customer_id, order_id)
        .await
        .unwrap();

    // verify order status
    let order = sqlx::query!(
        r#"
        SELECT status as "status: OrderStatus"
        FROM orders
        WHERE id = $1
        "#,
        order_id
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();

    assert_eq!(order.status, OrderStatus::COMPLETED);

    // verify detailer profile updated
    let detailer = sqlx::query!(
        r#"
        SELECT availability_status as "availability_status: DetailerAvailability", 
               total_jobs_completed
        FROM detailer_profiles
        WHERE user_id = $1
        "#,
        detailer_id
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();

    assert_eq!(detailer.total_jobs_completed, Some(11));

    // verify customer loyalty points updated
    let customer = sqlx::query!(
        r#"
        SELECT loyalty_points
        FROM customer_profiles
        WHERE user_id = $1
        "#,
        customer_id
    )
    .fetch_one(&test_db.pool)
    .await
    .unwrap();

    assert_eq!(customer.loyalty_points, Some(6));

    test_db.cleanup().await;
}