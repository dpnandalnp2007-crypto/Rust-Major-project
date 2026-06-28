use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    state::AppState,
    utils::{api_key_gen::hash_api_key, jwt::verify_access_token},
};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
            stripped.trim()
        } else {
            return Err(AppError::Unauthorized("Invalid Authorization header format. Expected Bearer <token>".to_string()));
        };

        let claims = verify_access_token(token, &state.config.jwt_secret)?;

        Ok(AuthUser {
            id: claims.sub,
            email: claims.email,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub organization_id: Uuid,
    pub scopes: Vec<String>,
}

#[async_trait]
impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let key_header = parts
            .headers
            .get("x-api-key")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing X-API-Key header".to_string()))?;

        let hash = hash_api_key(key_header);
        let api_key = state.repo.find_api_key_by_hash(&hash).await?
            .ok_or_else(|| AppError::Unauthorized("Invalid or revoked API key".to_string()))?;

        Ok(ApiKeyAuth {
            organization_id: api_key.organization_id,
            scopes: api_key.scopes,
        })
    }
}
