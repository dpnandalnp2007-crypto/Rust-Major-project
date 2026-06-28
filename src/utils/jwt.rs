use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub session_id: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

pub fn create_access_token(user_id: Uuid, email: &str, secret: &str, expire_minutes: i64) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let exp = now + (expire_minutes * 60);

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp,
        iat: now,
        token_type: "access".to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("Failed to create access token: {}", e)))
}

pub fn create_refresh_token(user_id: Uuid, session_id: Uuid, secret: &str, expire_days: i64) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let exp = now + (expire_days * 86400);

    let claims = RefreshClaims {
        sub: user_id,
        session_id,
        exp,
        iat: now,
        token_type: "refresh".to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {}", e)))
}

pub fn verify_access_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    if token_data.claims.token_type != "access" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    Ok(token_data.claims)
}

pub fn verify_refresh_token(token: &str, secret: &str) -> Result<RefreshClaims, AppError> {
    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    if token_data.claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    Ok(token_data.claims)
}
