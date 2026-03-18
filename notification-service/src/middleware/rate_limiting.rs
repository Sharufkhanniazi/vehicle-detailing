use axum::{
    extract::State,
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use redis::AsyncCommands;

use crate::state::AppState;

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {

    let user_id = request.extensions().get::<Uuid>().copied();

    if let Some(user_id) = user_id {
        let key = format!("rate_limit:{}", user_id);

        // get redis connection
        let mut conn = state
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let current_count: u32 = conn
            .get::<_, u32>(&key)
            .await
            .map_err(|e| {
                tracing::warn!("Rate limit read error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if current_count >= 100 {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        let _: () = conn
            .set_ex(&key, current_count + 1, 60)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(next.run(request).await)
}