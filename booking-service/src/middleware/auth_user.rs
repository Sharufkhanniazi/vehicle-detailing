use axum::{
    extract::{FromRef, FromRequestParts}, http::request::Parts
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader
};
use crate::state::AppState;
use crate::utils::error::AppError;
use uuid::Uuid;
use shared_auth::models::{Claims, UserRole};
use shared_auth::jwt::validate_token;


#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: UserRole
}

impl<S> FromRequestParts<S> for AuthUser 
where 
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;    

    async fn from_request_parts(
            parts: &mut Parts,
            state: &S,
        ) -> Result<Self, Self::Rejection> {
        
        let token = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map(|TypedHeader(Authorization(bearer))| bearer.token().to_string())
            .map_err(|_| AppError::InternalServerError("Unauthorized user".into()))?;

        let app_state = AppState::from_ref(state);
        
        let claims: Claims = validate_token(&token, app_state.jwt_secret.as_ref())
            .map_err(|_| AppError::InternalServerError("Invalid or expired token".into()))?;
        
        tracing::info!("{}({:?}) made a request", claims.username, claims.role);

        // store user ID in request extension for middleware access
        parts.extensions.insert(claims.sub);

        Ok(Self { 
            id: claims.sub,
            role: claims.role 
        })
    }
}

