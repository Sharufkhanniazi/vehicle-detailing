use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use shared_auth::models::UserRole;
use std::time::Instant;
use crate::utils::models::{EstimatePriceRequest, BookingRequest, CancelBooking, OrderCompleted, SubmitReviewRequest};
use crate::middleware::auth_user::AuthUser;
use crate::utils::error::{Result, AppError};
use crate::state::AppState;
use crate::services::metrics::{
    gather_metrics,
    HTTP_REQUESTS_TOTAL,
    HTTP_REQUEST_DURATION,
    HTTP_RESPONSE_SIZE,
    HTTP_RESPONSE_STATUS
};

pub async fn get_price(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<EstimatePriceRequest>
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["GET", "/price"])
        .inc();

    if auth_user.role != UserRole::CUSTOMER {

        HTTP_RESPONSE_STATUS
            .with_label_values(&["/price", "500"])
            .inc();

        return Err(AppError::InternalServerError("API is only for customers".into()));
    }

    let response = state.booking_service.get_price(request).await?;

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["GET", "/price"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/price", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/price"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn create_booking(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<BookingRequest>
)-> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/booking"])
        .inc();

    if auth_user.role != UserRole::CUSTOMER {

        HTTP_RESPONSE_STATUS
            .with_label_values(&["/booking", "500"])
            .inc();

        return Err(AppError::InternalServerError("API is only for customers".into()));
    }
    
    let response = state
        .booking_service
        .create_booking(auth_user.id, request)
        .await?;

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/booking"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/booking", "200"])
        .inc();

    let body = serde_json::to_string(&response)
        .map_err(|_| AppError::InternalServerError("Serialization Error".into()))?;

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/booking"])
        .inc_by(body.len() as u64);

    Ok(Json(response))
}

pub async fn cancel_booking(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<CancelBooking>
)-> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/cancel"])
        .inc();

    let role = auth_user.role.clone();

    state.booking_service
        .cancel_booking(request.order_id, auth_user.id, Some(role))
        .await?;

    let response = json!({
        "message": "Booking cancelled successfully",
        "order_id": request.order_id,
        "cancelled_by": auth_user.id,
        "role": auth_user.role
    });

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/cancel"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/cancel", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/cancel"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn order_completed(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<OrderCompleted>
)-> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/completed"])
        .inc();

    if auth_user.role != UserRole::DETAILER {

        HTTP_RESPONSE_STATUS
            .with_label_values(&["/completed", "500"])
            .inc();

        return Err(AppError::InternalServerError("API is only for detailer".into()));
    }

    state.booking_service
        .order_completed(auth_user.id, request.customer_id, request.order_id)
        .await?;

    let response = json!({
        "message": "Order Completed Successfully",
        "order_id": request.order_id,
        "Detailer_id": auth_user.id,
    });

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/completed"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/completed", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/completed"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn submit_review_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<SubmitReviewRequest>,
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/review"])
        .inc();

    if auth_user.role != UserRole::CUSTOMER {

        HTTP_RESPONSE_STATUS
            .with_label_values(&["/review", "500"])
            .inc();

        return Err(AppError::InternalServerError("API is only for customers".into()));
    }

    if request.rating < 1 || request.rating > 5 {

        HTTP_RESPONSE_STATUS
            .with_label_values(&["/review", "400"])
            .inc();

        return Err(AppError::InternalServerError("Bad Request".into()));
    }

    state.booking_service.submit_review(
        request.order_id,
        auth_user.id,
        request.detailer_id,
        request.rating,
        request.comment,
    )
    .await?;

    let response = json!({
        "Message": "Review Submitted Successfully",
        "order_id": request.order_id,
    });

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/review"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/review", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/review"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn metrics_handler() -> impl IntoResponse {
    gather_metrics()
}