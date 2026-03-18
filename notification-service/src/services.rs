use axum::extract::ws::{Message, Utf8Bytes};
use dashmap::DashMap;
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;
use serde::Serialize;
use std::sync::Arc;
use crate::util::send_push_notification_v1;
use crate::errors::Result;

#[derive(Serialize)]
pub struct WsNotification {
    pub title: String,
    pub body: String,
}

pub struct NotificationService;

impl NotificationService {
    // send WS notification if connected
    pub async fn send_ws_notification(
        connections: &Arc<DashMap<Uuid, Vec<mpsc::UnboundedSender<Message>>>>,
        user_id: Uuid,
        title: &str,
        body: &str
    ) {
        if let Some(entry) = connections.get(&user_id) {
            let message_str = serde_json::to_string(&WsNotification {
                title: title.to_string(),
                body: body.to_string(),
            }).unwrap();

            tracing::info!("Notification send through ws");

            for sender in entry.value() {
                let _ = sender.send(Message::Text(Utf8Bytes::from(message_str.clone())));
            }
        }
    }

    // send push notification via FCM
    pub async fn send_push(
        user_id: Uuid,
        pool: &PgPool,
        project_id: &str,
        title: &str,
        body: &str
    ) {

        let fcm_token_option = sqlx::query!(
            r#"
            SELECT fcm_token FROM users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await
        .unwrap();

        if let Some(record) = fcm_token_option {

            if let Some(token) = record.fcm_token {
            
                let _ = send_push_notification_v1(
                    &token,
                    title,
                    body,
                    project_id
                ).await;
                tracing::info!("Push notfication send");
            }
        }
        
    }

    // store notification in PostgreSQL
    pub async fn save_notification(
        pool: &PgPool,
        user_id: Uuid,
        title: &str,
        body: &str
    ) -> Result<()> {

       
        sqlx::query!(
            r#"
            INSERT INTO notifications (user_id, title, body)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            title,
            body
        )
        .execute(pool)
        .await?;

        tracing::info!("Notification saved to db");
        
        Ok(())
    }
}
