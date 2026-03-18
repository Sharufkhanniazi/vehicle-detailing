use sqlx::PgPool;
use std::env;
use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng, PasswordHash, PasswordVerifier}};
use shared_auth::models::{UserRole, UserCreatedEvent};
use crate::{utils::jwt::JwtService};
use crate::models::{RegisterUser, LoginUser, User, LoginResponse};
use crate::utils::error::{AppError, Result};
use crate::services::kafka_producer::KafkaProducer;

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    kafka: KafkaProducer,
    jwt_service: JwtService,
}

impl AuthService {

    pub fn new(pool: PgPool) -> Self {

        let jwt_service = JwtService::new();

        let kafka_broker = env::var("KAFKA_BROKER")
            .expect("KAFKA_BROKER must be set in .env");

        let kafka = KafkaProducer::new(&kafka_broker);

        AuthService { pool, kafka , jwt_service}
    }

    // helper fn
    fn hash_password(&self, password: &str) -> Result<String> {
        
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::HashPasswordError)?
            .to_string();

        Ok(password_hash)
    }

    pub async fn register(&self, user_data: RegisterUser) -> Result<String> {

        let mut tx = self.pool.begin().await?;
        let password_hash = self.hash_password(&user_data.password)?;

        // Check if user with the same email already exists
        let existing = sqlx::query!(
            "SELECT id, is_email_verified FROM users WHERE email = $1",
            user_data.email
        ).fetch_optional(&mut *tx)
        .await?;
        
        // If user exists  
        if let Some(record) = existing {
            // If email is already verified
            if record.is_email_verified {
                return Err(AppError::EmailAlreadyExists(user_data.email));
            } else {

                // Update the existing user's username, password hash and role
                sqlx::query!(
                    "UPDATE users SET username = $1, password_hash = $2, role = $3 WHERE id = $4",
                    user_data.username,
                    password_hash,
                    user_data.user_role as _,
                    record.id
                ).execute(&mut *tx).await?;

                // If row with this user_id already exists, do nothing instead of throwing an error, 
                match user_data.user_role {
                    UserRole::CUSTOMER => {
                        // Remove detailer profile if exists (data integrity)
                        sqlx::query!(
                            "DELETE FROM detailer_profiles WHERE user_id = $1",
                            record.id
                        )
                        .execute(&mut *tx)
                        .await?;
                    
                        // Insert customer profile
                        sqlx::query!(
                            "INSERT INTO customer_profiles (user_id) 
                             VALUES ($1) 
                             ON CONFLICT (user_id) DO NOTHING",
                            record.id
                        )
                        .execute(&mut *tx)
                        .await?;
                    }
                
                    UserRole::DETAILER => {
                        // Remove customer profile if exists (data integrity)
                        sqlx::query!(
                            "DELETE FROM customer_profiles WHERE user_id = $1",
                            record.id
                        )
                        .execute(&mut *tx)
                        .await?;
                    
                        // Insert detailer profile
                        sqlx::query!(
                            "INSERT INTO detailer_profiles (user_id) 
                             VALUES ($1) 
                             ON CONFLICT (user_id) DO NOTHING",
                            record.id
                        )
                        .execute(&mut *tx)
                        .await?;
                    }
                }

                tx.commit().await?;

                tracing::info!("Updated existing unverified user: {}", user_data.email);

                let token = self.jwt_service.generate_email_verification_token(record.id)?;

                let event = UserCreatedEvent {
                    user_id: record.id,
                    email: user_data.email.clone(),
                    email_token: token
                };

                self.kafka.send_user_created(event).await?;

                return Ok(format!("Registration successful. Verification mail send to {}.",user_data.email));
            }
        }

        let user = sqlx::query!(
            r#"
            INSERT INTO users (username, email, password_hash, role) 
            VALUES ($1, $2, $3, $4) 
            RETURNING id, email
            "#,
            user_data.username,
            user_data.email,
            password_hash,
            user_data.user_role as _
        ).fetch_one(&mut *tx)
        .await?;

        tracing::info!("New {:?} inserted into users table: {}", user_data.user_role, user.email);

        match user_data.user_role {
            UserRole::CUSTOMER => {
                sqlx::query!(
                    "INSERT INTO customer_profiles (user_id) VALUES ($1)",
                    user.id
                ).execute(&mut *tx)
                .await?;
            }
            UserRole::DETAILER => {
                sqlx::query!(
                    "INSERT INTO detailer_profiles (user_id) VALUES ($1)",
                    user.id
                ).execute(&mut *tx) 
                .await?;
            }
        }

        tx.commit().await?;

        let token = self.jwt_service.generate_email_verification_token(user.id)?;

        let event = UserCreatedEvent {
            user_id: user.id,
            email: user.email.clone(),
            email_token: token
        };

        self.kafka.send_user_created(event).await?;

        Ok(format!("Registration successful. Verification mail send to {}.",user_data.email))
    }

    pub async fn login(&self, login_data: LoginUser) -> Result<LoginResponse> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, 
            role as "role: UserRole", is_email_verified,
            is_active, created_at, updated_at
            FROM users 
            WHERE username = $1
            "#,
            login_data.username
        ).fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

        if !user.is_email_verified {
            return Err(AppError::UnverifiedUser);
        }

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::HashPasswordError)?;

        Argon2::default()
            .verify_password(login_data.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::HashPasswordError)?;

        let token = self.jwt_service.generate_token(
            user.id.clone(), 
            &user.email, 
            &user.username, 
            user.role.clone()
        )?;

        tracing::info!("{} logged in", user.username);

        let response = LoginResponse {
            user,
            token
        };

        Ok(response)
    }

    pub async fn resend_email_verification_token(&self, email: &str) -> Result<String> {

        let user = sqlx::query!(
            "SELECT id FROM users WHERE email = $1",
            email
        ).fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

        let token = self.jwt_service.generate_email_verification_token(user.id)?;

        let event = UserCreatedEvent {
            user_id: user.id,
            email: email.to_string(),
            email_token: token
        };

        self.kafka.send_user_created(event).await?;

        tracing::info!("Email verification request send");

        Ok(format!("Verfication mail sent to {}", email))
    }

    pub async fn verify_email(&self, token: &str) -> Result<String> {
        
        match self.jwt_service.validate_email_verification_token(token) {
            Ok(email_claims) => {
                
                sqlx::query!(
                    "UPDATE users SET is_email_verified = true WHERE id = $1",
                    email_claims.sub
                )
                .execute(&self.pool)
                .await?;

                tracing::info!("Email verified for user ID in db: {}", email_claims.sub);

                Ok("Email verified successfully".into())
                
            }
            Err(e) => Err(AppError::InternalServerError(format!("{}",e))),
        }
    }

}