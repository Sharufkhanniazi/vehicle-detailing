use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    extract::State,
    response::IntoResponse
};
use tokio::sync::mpsc;
use uuid::Uuid;
use futures::{SinkExt, StreamExt};
use crate::state::AppState;
use crate::middleware::auth_user::AuthUser;

pub async fn ws_notifications(
    ws: WebSocketUpgrade,
    auth_user:AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(state, socket, auth_user.id))
}

async fn handle_ws(state: AppState, socket: WebSocket, user_id: Uuid) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (mut sender, mut receiver) = socket.split();

    // Save connection
    state.ws_connection
        .entry(user_id)
        .or_insert_with(Vec::new)
        .push(tx);

    // Keepalive  receive loop
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                let _ = sender.send(msg).await;
            }

            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    Message::Ping(v) => { 
                        let _ = sender.send(Message::Pong(v)).await; 
                    }

                    Message::Close(_) => break,
                    
                    _ => {}
                }
            }

            else => break
        }
    }

    // Cleanup
    if let Some(mut entry) = state.ws_connection.get_mut(&user_id) {
        let senders = entry.value_mut();
        senders.retain(|s| !s.is_closed());
        if senders.is_empty() {
            drop(entry);
            state.ws_connection.remove(&user_id);
        }
    }
}