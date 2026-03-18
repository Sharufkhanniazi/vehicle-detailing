use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn send_email_verification(email: &str, token: &str) -> anyhow::Result<()> {

    let resend_api_key = env::var("RESEND_API_KEY")
        .expect("Failed to read RESEND_API_KEY from .env");

    let verification_url = format!("http://localhost:3000/verify-email?token={}", token);

    let body = json!({
        "from": "onboarding@resend.dev",
        "to": email,
        "subject": "Verify your email address",
        "html": format!(
            "<p>Click the link below to verify your email address:</p><p>
            <a href=\"{}\">Verify Email</a></p>",
            verification_url
        )
    });

    Client::new() 
        .post("https://api.resend.com/emails") 
        .bearer_auth(resend_api_key) 
        .json(&body) 
        .send() 
        .await?; 

    Ok(())
}