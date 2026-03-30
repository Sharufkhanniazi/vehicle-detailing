use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use uuid::Uuid;
use url::Url;
use assign_detailer_service::assign_detailer::DetailerAssignmentService;
use assign_detailer_service::kafka_producer::KafkaProducer;

pub struct TestDb {
    pub pool: PgPool,
    db_name: String,
}

impl TestDb {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let base_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let db_name = format!("test_db_{}", Uuid::new_v4());

        // Create test database
        let admin_pool = PgPoolOptions::new()
            .connect(&base_url)
            .await
            .unwrap();

        sqlx::query(&format!(r#"CREATE DATABASE "{}""#, db_name))
            .execute(&admin_pool)
            .await
            .unwrap();

        // Connect to test database
        let mut url: Url = base_url.parse().unwrap();
        url.set_path(&format!("/{}", db_name));
        let test_db_url = url.to_string();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&test_db_url)
            .await
            .unwrap();

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        Self { pool, db_name }
    }

    pub async fn cleanup(&self) {
        self.pool.close().await;

        let admin_url = env::var("DATABASE_URL").unwrap();
        let admin_pool = PgPoolOptions::new()
            .connect(&admin_url)
            .await
            .unwrap();

        sqlx::query(&format!(
            r#"
            SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = '{}'
            "#,
            self.db_name
        ))
        .execute(&admin_pool)
        .await
        .unwrap();

        sqlx::query(&format!(r#"DROP DATABASE "{}""#, self.db_name))
            .execute(&admin_pool)
            .await
            .unwrap();
    }
}

pub async fn create_test_service() -> (DetailerAssignmentService, TestDb) {
    let test_db = TestDb::new().await;
    let kafka = KafkaProducer::new("localhost:9092");
    
    let service = DetailerAssignmentService {
        pool: test_db.pool.clone(),
        kafka,
    };
    
    (service, test_db)
}