use prometheus::{
    Encoder, HistogramVec, IntCounterVec, TextEncoder, register_histogram_vec,
    register_int_counter_vec
};
use lazy_static::lazy_static;
use crate::utils::errors::{AppError, Result};

lazy_static! {

    // Total HTTP Requests
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["method", "endpoint"]
    ).expect("Failed to register http_requests_total metric");

    // Success / Failure counter
    pub static ref HTTP_RESPONSE_STATUS: IntCounterVec = register_int_counter_vec!(
        "http_response_status_total",
        "Total responses by status",
        &["endpoint", "status"]
    ).expect("Failed to register http_requests_total metric");

    //  Request latency
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "Request latency",
        &["method", "endpoint"]
    ).expect("Failed to register http_requests_total metric");

    //  Response size
    pub static ref HTTP_RESPONSE_SIZE: IntCounterVec = register_int_counter_vec!(
        "http_response_size_bytes",
        "Response size",
        &["endpoint"]
    ).expect("Failed to register http_requests_total metric");
}

pub fn gather_metrics() -> Result<String> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();

    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let metrics = String::from_utf8(buffer)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(metrics)
}