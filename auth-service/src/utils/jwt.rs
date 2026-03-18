use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;
use chrono::{Duration, Utc};
use shared_auth::models::{Claims, UserRole, EmailVerificationClaims};
use crate::utils::error::Result;

#[derive(Clone)]
pub struct JwtService {
    secret_key: String,
    jwt_expiration: usize,
}

impl JwtService {
    pub fn new() -> Self {
        dotenv().ok();

        let secret_key = env::var("JWT_SECRET_KEY").expect("Failed to read JWT_SECRET_KEY from .env");

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .expect("Failed to read JWT_EXPIRATION from .env")
            .parse::<usize>()
            .expect("JWT_EXPIRATION must be a valid number");

        JwtService {
            secret_key,
            jwt_expiration, 
        }
    }

    pub fn generate_token(&self, user_id: Uuid, email: &str, username: &str, role: UserRole) -> Result<String> {

        let expiration_time = Utc::now()
            .checked_add_signed(Duration::hours(self.jwt_expiration as i64))
            .expect("Valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            username: username.to_string(),
            exp: expiration_time,
            role
        };

        let token = encode(
                &Header::new(Algorithm::HS256), 
                &claims, 
                &EncodingKey::from_secret(self.secret_key.as_ref())
            )?;

        Ok(token)
    }


    pub fn generate_email_verification_token(&self, user_id: Uuid) -> Result<String> {

        let expiration_time = Utc::now()
            .checked_add_signed(Duration::minutes(30))
            .expect("Valid timestamp")
            .timestamp() as usize;

        let claims = EmailVerificationClaims {
            sub: user_id,
            exp: expiration_time // 30 minutes expiration
        };

        let token = encode(
            &Header::new(Algorithm::HS256), 
            &claims, 
            &EncodingKey::from_secret(self.secret_key.as_ref())
        )?;

        Ok(token)
    }

    pub fn validate_email_verification_token(&self, token: &str) -> Result<EmailVerificationClaims> {
        let token_data = decode::<EmailVerificationClaims>(
            token, 
            &DecodingKey::from_secret(self.secret_key.as_ref()), 
            &Validation::new(Algorithm::HS256)
        )?;

        Ok(token_data.claims)
    }

}


