use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::models::Claims;

pub fn validate_token(token: &str, secret: &str) -> jsonwebtoken::errors::Result<Claims> {

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256)
    )?;

    Ok(token_data.claims)
}