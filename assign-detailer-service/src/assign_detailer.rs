use uuid::Uuid;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use chrono::{DateTime, Utc};
use crate::kafka_producer::KafkaProducer;
use crate::errors::Result;

pub struct DetailerAssignmentService {
    pub pool: PgPool,
    pub kafka: KafkaProducer,
}

impl DetailerAssignmentService {

    pub async fn new()-> Result<Self> {

        let db_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env");

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await?;

        let kafka_broker = env::var("KAFKA_BROKER")
            .expect("KAFKA_BROKER must be set in .env");

        let kafka = KafkaProducer::new(&kafka_broker);

        Ok(DetailerAssignmentService { 
            pool, 
            kafka 
        })
    }

    pub async fn assign_detailer(
            &self,
            latitude: f64,
            longitude: f64,
            time_slot: DateTime<Utc> 
        ) -> Result<Option<Uuid>> {

        // Find available detailers based on proximity and current availability
        let available_detailer = sqlx::query!(
            r#"
            SELECT id
            FROM (
                SELECT
                    u.id,
                    dp.rating,
                    dp.total_jobs_completed,
                    dp.availability_status as "availability_status: AvailabilityStatus",
                    (
                        6371 * acos(
                            cos(radians($1)) *
                            cos(radians(dp.last_known_latitude)) *
                            cos(radians(dp.last_known_longitude) - radians($2)) +
                            sin(radians($1)) *
                            sin(radians(dp.last_known_latitude))
                        )
                    ) AS distance_km
                FROM users u
                JOIN detailer_profiles dp ON u.id = dp.user_id
                WHERE
                    u.role = 'DETAILER'
                    AND u.is_active = true
                    AND dp.availability_status = 'ONLINE'
                    AND dp.last_known_latitude BETWEEN ($1 - 0.09) AND ($1 + 0.09)
                    AND dp.last_known_longitude BETWEEN ($2 - 0.09) AND ($2 + 0.09)
                    AND NOT EXISTS (
                        SELECT 1
                        FROM orders o2
                        WHERE o2.detailer_id = u.id
                        AND o2.time_slot = $3
                        AND o2.status NOT IN ('COMPLETED','CANCELLED')
                    )
            ) AS candidates
            WHERE distance_km <= 10
            ORDER BY distance_km ASC,
                     rating DESC NULLS LAST,
                     total_jobs_completed DESC
            LIMIT 1;
            "#,
            latitude,   // $1
            longitude,  // $2
            time_slot   // $3
        )
        .fetch_optional(&self.pool)
        .await?;

        let detailer_id = match available_detailer {
            Some(detailer) => detailer.id,
            None => { return Ok(None); }
        };

        Ok(Some(detailer_id))
    }
}