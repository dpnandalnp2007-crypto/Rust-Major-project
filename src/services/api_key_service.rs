use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::CreateApiKeyRequest,
    error::AppError,
    models::{ApiKey, ApiKeyCreatedResponse, ApiKeyResponse, AuditLog},
    repositories::AppRepository,
    utils::api_key_gen::generate_api_key,
};

pub struct ApiKeyService;

impl ApiKeyService {
    pub async fn create(
        repo: Arc<dyn AppRepository>,
        req: CreateApiKeyRequest,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<ApiKeyCreatedResponse, AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::Validation("API Key name cannot be empty".to_string()));
        }

        let (raw_key, key_prefix, key_hash) = generate_api_key();
        let now = Utc::now();
        let key_id = Uuid::new_v4();

        let api_key = ApiKey {
            id: key_id,
            organization_id: req.organization_id,
            name: req.name.clone(),
            description: req.description.clone(),
            key_prefix,
            key_hash,
            scopes: req.scopes.clone(),
            status: "active".to_string(),
            expires_at: req.expires_at,
            created_at: now,
        };

        let created = repo.create_api_key(api_key).await?;

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: Some(req.organization_id),
            actor_id: Some(actor_id),
            action: "api_key.create".to_string(),
            resource: format!("api_key:{}", created.id),
            details: Some(serde_json::json!({ "name": created.name, "scopes": created.scopes })),
            ip_address: ip,
            created_at: now,
        }).await;

        Ok(ApiKeyCreatedResponse {
            id: created.id,
            organization_id: created.organization_id,
            name: created.name,
            description: created.description,
            raw_key,
            scopes: created.scopes,
            expires_at: created.expires_at,
            created_at: created.created_at,
        })
    }

    pub async fn list(repo: Arc<dyn AppRepository>, org_id: Uuid) -> Result<Vec<ApiKeyResponse>, AppError> {
        let keys = repo.list_org_api_keys(org_id).await?;
        Ok(keys.into_iter().map(Into::into).collect())
    }

    pub async fn revoke(
        repo: Arc<dyn AppRepository>,
        id: Uuid,
        org_id: Uuid,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<(), AppError> {
        repo.revoke_api_key(id, org_id).await?;

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: Some(org_id),
            actor_id: Some(actor_id),
            action: "api_key.revoke".to_string(),
            resource: format!("api_key:{}", id),
            details: None,
            ip_address: ip,
            created_at: Utc::now(),
        }).await;

        Ok(())
    }
}
