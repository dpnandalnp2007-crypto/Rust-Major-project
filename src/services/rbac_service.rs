use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{CreatePermissionRequest, CreateRoleRequest, EvaluatePermissionResponse, UpdateRoleRequest},
    error::AppError,
    models::{AuditLog, Permission, Role, RoleWithPermissions},
    repositories::AppRepository,
};

pub struct RbacService;

impl RbacService {
    pub async fn create_role(
        repo: Arc<dyn AppRepository>,
        org_id: Option<Uuid>,
        req: CreateRoleRequest,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<Role, AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::Validation("Role name cannot be empty".to_string()));
        }

        let now = Utc::now();
        let role_id = Uuid::new_v4();

        let role = Role {
            id: role_id,
            organization_id: org_id,
            name: req.name.clone(),
            description: req.description,
            is_system: false,
            created_at: now,
        };

        let created_role = repo.create_role(role).await?;

        // Assign permissions
        for pname in &req.permission_names {
            if let Some(perm) = repo.find_permission_by_name(pname).await? {
                repo.assign_permission_to_role(created_role.id, perm.id).await?;
            } else {
                return Err(AppError::NotFound(format!("Permission '{}' does not exist", pname)));
            }
        }

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: org_id,
            actor_id: Some(actor_id),
            action: "role.create".to_string(),
            resource: format!("role:{}", created_role.id),
            details: Some(serde_json::json!({ "name": created_role.name, "permissions": req.permission_names })),
            ip_address: ip,
            created_at: now,
        }).await;

        Ok(created_role)
    }

    pub async fn list_roles(repo: Arc<dyn AppRepository>, org_id: Option<Uuid>) -> Result<Vec<RoleWithPermissions>, AppError> {
        repo.list_roles(org_id).await
    }

    pub async fn update_role(
        repo: Arc<dyn AppRepository>,
        role_id: Uuid,
        req: UpdateRoleRequest,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<Role, AppError> {
        let role = repo.update_role(role_id, req.name.as_deref(), req.description.as_deref()).await?;

        if let Some(pnames) = &req.permission_names {
            repo.remove_permissions_from_role(role_id).await?;
            for pname in pnames {
                if let Some(perm) = repo.find_permission_by_name(pname).await? {
                    repo.assign_permission_to_role(role_id, perm.id).await?;
                } else {
                    return Err(AppError::NotFound(format!("Permission '{}' does not exist", pname)));
                }
            }
        }

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: role.organization_id,
            actor_id: Some(actor_id),
            action: "role.update".to_string(),
            resource: format!("role:{}", role.id),
            details: Some(serde_json::json!({ "name": role.name })),
            ip_address: ip,
            created_at: Utc::now(),
        }).await;

        Ok(role)
    }

    pub async fn delete_role(
        repo: Arc<dyn AppRepository>,
        role_id: Uuid,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<(), AppError> {
        let role = repo.find_role_by_id(role_id).await?;
        repo.delete_role(role_id).await?;

        if let Some(r) = role {
            let _ = repo.log_audit(AuditLog {
                id: Uuid::new_v4(),
                organization_id: r.organization_id,
                actor_id: Some(actor_id),
                action: "role.delete".to_string(),
                resource: format!("role:{}", role_id),
                details: Some(serde_json::json!({ "name": r.name })),
                ip_address: ip,
                created_at: Utc::now(),
            }).await;
        }

        Ok(())
    }

    pub async fn create_permission(repo: Arc<dyn AppRepository>, req: CreatePermissionRequest) -> Result<Permission, AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::Validation("Permission name cannot be empty".to_string()));
        }

        let perm = Permission {
            id: Uuid::new_v4(),
            name: req.name.to_lowercase(),
            description: req.description,
            created_at: Utc::now(),
        };

        repo.create_permission(perm).await
    }

    pub async fn list_permissions(repo: Arc<dyn AppRepository>) -> Result<Vec<Permission>, AppError> {
        repo.list_permissions().await
    }

    pub async fn evaluate(
        repo: Arc<dyn AppRepository>,
        user_id: Uuid,
        org_id: Uuid,
        permission: &str,
    ) -> Result<EvaluatePermissionResponse, AppError> {
        let allowed = repo.has_permission(user_id, org_id, permission).await?;
        Ok(EvaluatePermissionResponse {
            allowed,
            user_id,
            organization_id: org_id,
            permission: permission.to_string(),
        })
    }
}
