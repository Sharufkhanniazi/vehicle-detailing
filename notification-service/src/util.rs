use yup_oauth2::read_service_account_key;
use yup_oauth2::ServiceAccountAuthenticator;
use reqwest::Client;
use serde_json::json;
use crate::errors::{Result, AppError};

pub async fn send_push_notification_v1(
    fcm_token: &str,
    title: &str,
    body: &str,
    project_id: &str
) -> Result<()> {

    // Load service account and get OAuth token 
    let sa_key = read_service_account_key(
        "vehicle_detailing.json"
    ).await
    .map_err(|e| AppError::InternalServerError(format!("Failed to read service account: {}", e)))?;

    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await
        .map_err(|e| AppError::InternalServerError(format!("Failed to build authenticator: {}", e)))?;

    let scopes = &["https://www.googleapis.com/auth/firebase.messaging"];
    let token = auth.token(scopes).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to get token: {}", e)))?;

    let token_str = token.token()
        .ok_or_else(|| AppError::InternalServerError("Failed to get OAuth token - token is None".into()))?;
    
    send_fcm_message(
        token_str,  
        project_id, 
        fcm_token, 
        title, 
        body
    ).await?;

    Ok(())
}

pub async fn send_fcm_message(
    oauth_token: &str,
    project_id: &str,
    fcm_token: &str,
    title: &str,
    body: &str
) -> Result<()> {

    let client = Client::new();
    let url = format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send",
        project_id
    );

    let payload = json!({
        "message": {
            "token": fcm_token,
            "notification": {
                "title": title,
                "body": body
            },
            "android": { "priority": "high" },
            "apns": { "headers": { "apns-priority": "10" } }
        }
    });

    // ✅ FIX 2: This should work once json feature is enabled
    let response = client.post(&url)
        .header("Authorization", format!("Bearer {}", oauth_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    
    response.error_for_status()?;
    
    Ok(())
}