use axum::{
    Json,
    response::IntoResponse,
    extract::{ws::{WebSocketUpgrade, WebSocket, Message, Utf8Bytes}, Path, Query, State},
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;
use tokio::sync::mpsc;
use shared_auth::models::{UserRole, DetailerArrivedEvent};
use crate::utils::models::{LocationUpdate, WsMessage, DistanceQuery};
use crate::state::AppState;
use crate::services::tracking::TrackingService;
use crate::utils::models::DistanceResponse;
use crate::middleware::auth_user::AuthUser;
use crate::utils::errors::{Result, AppError};
use crate::services::metrics::{
    gather_metrics,
    HTTP_REQUESTS_TOTAL,
    HTTP_REQUEST_DURATION,
    HTTP_RESPONSE_SIZE,
    HTTP_RESPONSE_STATUS
};

pub async fn update_location_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(update): Json<LocationUpdate>
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
    .with_label_values(&["POST", "/update-location"])
    .inc();

    if auth_user.role != UserRole::DETAILER {
        HTTP_RESPONSE_STATUS.with_label_values(&["/update-location", "500"]).inc();
        return Err(AppError::InternalServerError("Only detailers can update location".into()));
    }

    let detailer_id = auth_user.id;

    TrackingService::update_location(&state.pool, &state.redis, update.clone(), detailer_id).await?;

    if let Some(order_id) = update.order_id {
        if let Ok(info) = TrackingService::get_tracking_info(&state.pool, &state.redis, order_id, detailer_id).await {
            let message = WsMessage::LocationUpdate {
                order_id,
                latitude: update.latitude,
                longitude: update.longitude,
                distance_km: info.distance_km,
                eta_minutes: info.eta_minutes,
                timestamp: update.timestamp,
            };
            TrackingService::broadcast_to_order(&state.active_connections, order_id, message).await;
        }
    }

    let response = json!({"status": "ok"});

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/update-location"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/update-location", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/update-location"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn get_tracking_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["GET", "/tracking"])
        .inc();

    let info = TrackingService::get_tracking_info(&state.pool, &state.redis, order_id, auth_user.id).await?;

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["GET", "/tracking"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/tracking", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/tracking"])
        .inc_by(std::mem::size_of_val(&info) as u64);

    Ok(Json(info))
}

pub async fn calculate_distance_handler(
    Query(params): Query<DistanceQuery>
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["GET", "/calculate-distance"])
        .inc();

    let distance_km = TrackingService::calculate_distance(params.lat1, params.lng1, params.lat2, params.lng2);
    let eta_minutes = TrackingService::calculate_eta(distance_km);

    let response = DistanceResponse { distance_km, eta_minutes };
    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["GET", "/calculate-distance"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/calculate-distance", "200"])
        .inc();

    HTTP_RESPONSE_SIZE
        .with_label_values(&["/calculate-distance"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn ws_tracking_handler(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    auth_user: AuthUser,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["GET", "/tracking/ws"])
        .inc();

    let user_id = auth_user.id;

    ws.on_upgrade(move |socket| handle_websocket(state, socket, order_id, user_id))
}

async fn handle_websocket(
    state: AppState,
    socket: WebSocket,
    order_id: Uuid,
    user_id: Uuid
) {
    
    let has_access = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM orders 
            WHERE id = $1 AND (customer_id = $2 OR detailer_id = $2)
        ) as "exists!"
        "#,
        order_id,
        user_id
    )
    .fetch_one(&state.pool)
    .await
    .map(|r| r.exists)
    .unwrap_or(false);

    if !has_access {
        return;
    }

    let (tx, mut rx) = mpsc::unbounded_channel();
    let (mut sender, mut receiver) = socket.split();

    state.active_connections
        .entry(order_id)
        .or_insert_with(Vec::new)
        .push(tx);

    if let Ok(info) = TrackingService::get_tracking_info(
        &state.pool, 
        &state.redis, 
        order_id, 
        user_id
    ).await {
        let msg = serde_json::to_string(&WsMessage::LocationUpdate { 
            order_id, 
            latitude: info.detailer_lat, 
            longitude: info.detailer_lng, 
            distance_km: info.distance_km, 
            eta_minutes: info.eta_minutes, 
            timestamp: info.last_updated
        }).unwrap();

        let _ = sender
            .send(Message::Text(Utf8Bytes::from(msg)))
            .await;
    }

    // main websocket loop 
    // The loop continues until:

    // the client disconnects
    // the channel closes
    // an error occurs
    // When something goes wrong, break stops the loop and the connection closes
    loop {
        // tokio::select! — wait for multiple async events
        // Whichever arrives first will run.
        tokio::select! {
            Some(msg) = rx.recv() => {
                if sender.send(msg).await.is_err() {
                    break;
                }
            }

            // WebSocket protocol requires responding to Ping with Pong.
            // Client: PING
            // Server responds: PONG
            // This keeps the connection alive.
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    Message::Ping(v) => {
                        let _ = sender.send(Message::Pong(v)).await;
                    }

                    Message::Close(_) => {
                        break;
                    }

                    // Detailer sends location via HTTP (update_location_handler), NOT via tx
                    // tx/rx are BOTH on the customer's side:
                    // tx = channel sender (stored in active_connections)
                    // rx = channel receiver (in customer's WebSocket handler)
                    // The flow:
                    // Detailer → HTTP → Server
                    // Server → finds customer's tx in active_connections
                    // Server → sends to customer's tx
                    // Customer's rx receives → sends via WebSocket → customer's phone

                    // The handle_websocket function ignores Message::Text and Message::Binary 
                    // because its only purpose is to send location updates to the client, 
                    // not to receive them from the client.
                    // The WebSocket is designed as one-way communication:
                    // Server → Client: Location updates (via rx.recv() channel)
                    // Client → Server: Only ping/pong (keepalive) and close messages

                    // Ignore other messages
                    // _ => {}
                    // Your server ignores things like:
                    // Text
                    // Binary
                    // Continuation
                    _ => {}
                }
            }
            // This runs when all streams are closed.
            // Examples:
            // rx channel closed
            // AND
            // websocket closed
            // Then the loop exits safely.
            else => break
        }
    }

    // When you call state.active_connections.get_mut(&order_id), 
    // it returns a special "guard" that holds a lock on that entry in the DashMap.
    if let Some(mut entry) = state.active_connections.get_mut(&order_id) {
        let senders = entry.value_mut();
        senders.retain(|s| !s.is_closed());
        if senders.is_empty() {
            // Release the lock manually
            drop(entry);
            state.active_connections.remove(&order_id);
        }
    }
}

pub async fn notify_arrival_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(order_id): Path<Uuid>
) -> Result<impl IntoResponse> {

    let timer = Instant::now();

    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/notify-arrival"])
        .inc();

    if auth_user.role != UserRole::DETAILER {
        HTTP_RESPONSE_STATUS
            .with_label_values(&["/notify-arrival", "500"])
            .inc();

        return Err(AppError::InternalServerError("Only detailers handler".into()));
    }

    let event = DetailerArrivedEvent { order_id, detailer_id: auth_user.id };

    state.kafka.detailer_arrived(event).await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let response = json!({"status": "arrival_notification_sent"});

    let duration = timer.elapsed().as_secs_f64();

    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/notify-arrival"])
        .observe(duration);

    HTTP_RESPONSE_STATUS
        .with_label_values(&["/notify-arrival", "200"])
        .inc();
    HTTP_RESPONSE_SIZE
        .with_label_values(&["/notify-arrival"])
        .inc_by(std::mem::size_of_val(&response) as u64);

    Ok(Json(response))
}

pub async fn metrics_handler() -> impl IntoResponse {
    gather_metrics()
}