use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use chrono::{DateTime, Utc};
use shared_auth::models::UserRole;

#[derive(Serialize, Deserialize, Validate)]
pub struct RegisterUser {
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    pub user_role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerification {
    pub email: String
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}