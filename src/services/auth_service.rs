use std::sync::Arc;
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    config::Config,
    dto::{AuthTokensResponse, LoginRequest, RefreshTokenRequest, RegisterRequest},
    error::AppError,
    models::{AuditLog, Session, User},
    repositories::AppRepository,
    utils::{
        jwt::{create_access_token, create_refresh_token, verify_refresh_token},
        password::{hash_password, verify_password},
    },
};

pub struct AuthService;

impl AuthService {
    pub async fn register(
        repo: Arc<dyn AppRepository>,
        config: &Config,
        req: RegisterRequest,
        ip: Option<String>,
    ) -> Result<AuthTokensResponse, AppError> {
        if req.password.len() < 8 {
            return Err(AppError::Validation("Password must be at least 8 characters long".to_string()));
        }

        let password_hash = hash_password(&req.password)?;
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let user = User {
            id: user_id,
            email: req.email.clone(),
            password_hash,
            full_name: req.full_name,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        };

        let created_user = repo.create_user(user).await?;

        // Create session
        let session_id = Uuid::new_v4();
        let refresh_token = create_refresh_token(
            created_user.id,
            session_id,
            &config.jwt_secret,
            config.refresh_token_expire_days,
        )?;
        let refresh_token_hash = hash_token(&refresh_token);

        let session = Session {
            id: session_id,
            user_id: created_user.id,
            refresh_token_hash,
            device_info: Some("Web Browser".to_string()),
            ip_address: ip.clone(),
            expires_at: now + chrono::Duration::days(config.refresh_token_expire_days),
            created_at: now,
        };

        repo.create_session(session).await?;

        let access_token = create_access_token(
            created_user.id,
            &created_user.email,
            &config.jwt_secret,
            config.access_token_expire_minutes,
        )?;

        // Audit log
        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: None,
            actor_id: Some(created_user.id),
            action: "user.register".to_string(),
            resource: format!("user:{}", created_user.id),
            details: Some(serde_json::json!({ "email": created_user.email })),
            ip_address: ip,
            created_at: now,
        }).await;

        Ok(AuthTokensResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_token_expire_minutes * 60,
        })
    }

    pub async fn login(
        repo: Arc<dyn AppRepository>,
        config: &Config,
        req: LoginRequest,
        ip: Option<String>,
    ) -> Result<AuthTokensResponse, AppError> {
        let user = repo
            .find_user_by_email(&req.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        if user.status != "active" {
            return Err(AppError::Forbidden("User account is suspended or deactivated".to_string()));
        }

        let is_valid = verify_password(&req.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Unauthorized("Invalid email or password".to_string()));
        }

        let now = Utc::now();
        let session_id = Uuid::new_v4();
        let refresh_token = create_refresh_token(
            user.id,
            session_id,
            &config.jwt_secret,
            config.refresh_token_expire_days,
        )?;
        let refresh_token_hash = hash_token(&refresh_token);

        let session = Session {
            id: session_id,
            user_id: user.id,
            refresh_token_hash,
            device_info: req.device_info.or_else(|| Some("Unknown Device".to_string())),
            ip_address: ip.clone(),
            expires_at: now + chrono::Duration::days(config.refresh_token_expire_days),
            created_at: now,
        };

        repo.create_session(session).await?;

        let access_token = create_access_token(
            user.id,
            &user.email,
            &config.jwt_secret,
            config.access_token_expire_minutes,
        )?;

        // Audit log
        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: None,
            actor_id: Some(user.id),
            action: "user.login".to_string(),
            resource: format!("user:{}", user.id),
            details: Some(serde_json::json!({ "session_id": session_id })),
            ip_address: ip,
            created_at: now,
        }).await;

        Ok(AuthTokensResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: config.access_token_expire_minutes * 60,
        })
    }

    pub async fn refresh(
        repo: Arc<dyn AppRepository>,
        config: &Config,
        req: RefreshTokenRequest,
    ) -> Result<AuthTokensResponse, AppError> {
        let _claims = verify_refresh_token(&req.refresh_token, &config.jwt_secret)?;
        let hash = hash_token(&req.refresh_token);

        let session = repo
            .find_session_by_hash(&hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Session revoked or expired".to_string()))?;

        let user = repo
            .find_user_by_id(session.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

        let access_token = create_access_token(
            user.id,
            &user.email,
            &config.jwt_secret,
            config.access_token_expire_minutes,
        )?;

        Ok(AuthTokensResponse {
            access_token,
            refresh_token: req.refresh_token, // Keep same refresh token
            token_type: "Bearer".to_string(),
            expires_in: config.access_token_expire_minutes * 60,
        })
    }

    pub async fn logout(
        repo: Arc<dyn AppRepository>,
        config: &Config,
        refresh_token: String,
    ) -> Result<(), AppError> {
        if let Ok(claims) = verify_refresh_token(&refresh_token, &config.jwt_secret) {
            let _ = repo.revoke_session(claims.session_id, claims.sub).await;
        }
        Ok(())
    }
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
