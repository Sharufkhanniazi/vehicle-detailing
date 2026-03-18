use axum::{
    http::StatusCode, 
    Json,
    response::{IntoResponse, Response}
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {  
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("HTTP request error: {0}")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", msg)),
            AppError::HttpRequestError(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("HTTP request error: {}", e)),
        };

        let body = Json(json!({ 
            "error": error_message,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;