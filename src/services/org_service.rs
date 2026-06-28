use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{AddMemberRequest, CreateOrgRequest, UpdateOrgRequest},
    error::AppError,
    models::{AuditLog, MemberDetail, Membership, Organization, PaginatedResponse, PaginationQuery},
    repositories::AppRepository,
};

pub struct OrgService;

impl OrgService {
    pub async fn create_org(
        repo: Arc<dyn AppRepository>,
        creator_id: Uuid,
        req: CreateOrgRequest,
        ip: Option<String>,
    ) -> Result<Organization, AppError> {
        if req.name.trim().is_empty() || req.slug.trim().is_empty() {
            return Err(AppError::Validation("Organization name and slug cannot be empty".to_string()));
        }

        let now = Utc::now();
        let org_id = Uuid::new_v4();

        let org = Organization {
            id: org_id,
            name: req.name.clone(),
            slug: req.slug.to_lowercase(),
            description: req.description,
            created_at: now,
            updated_at: now,
        };

        let created_org = repo.create_org(org).await?;

        // Create membership for creator
        let membership_id = Uuid::new_v4();
        let membership = Membership {
            id: membership_id,
            user_id: creator_id,
            organization_id: created_org.id,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        };
        repo.add_membership(membership).await?;

        // Assign default 'Owner' role to creator
        if let Some(owner_role) = repo.find_role_by_name(None, "Owner").await? {
            repo.assign_role_to_member(membership_id, owner_role.id).await?;
        }

        // Audit log
        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: Some(created_org.id),
            actor_id: Some(creator_id),
            action: "organization.create".to_string(),
            resource: format!("organization:{}", created_org.id),
            details: Some(serde_json::json!({ "name": created_org.name, "slug": created_org.slug })),
            ip_address: ip,
            created_at: now,
        }).await;

        Ok(created_org)
    }

    pub async fn get_org(repo: Arc<dyn AppRepository>, org_id: Uuid) -> Result<Organization, AppError> {
        repo.find_org_by_id(org_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))
    }

    pub async fn list_orgs(
        repo: Arc<dyn AppRepository>,
        query: &PaginationQuery,
    ) -> Result<PaginatedResponse<Organization>, AppError> {
        repo.list_orgs(query).await
    }

    pub async fn update_org(
        repo: Arc<dyn AppRepository>,
        org_id: Uuid,
        req: UpdateOrgRequest,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<Organization, AppError> {
        let updated = repo.update_org(org_id, req.name.as_deref(), req.description.as_deref()).await?;

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: Some(org_id),
            actor_id: Some(actor_id),
            action: "organization.update".to_string(),
            resource: format!("organization:{}", org_id),
            details: Some(serde_json::json!({ "name": updated.name })),
            ip_address: ip,
            created_at: Utc::now(),
        }).await;

        Ok(updated)
    }

    pub async fn add_member(
        repo: Arc<dyn AppRepository>,
        org_id: Uuid,
        req: AddMemberRequest,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<MemberDetail, AppError> {
        let user = repo
            .find_user_by_id(req.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Target user not found".to_string()))?;

        let now = Utc::now();
        let membership_id = Uuid::new_v4();
        let membership = Membership {
            id: membership_id,
            user_id: user.id,
            organization_id: org_id,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        };

        repo.add_membership(membership).await?;

        // Assign roles
        let role_names = if req.role_names.is_empty() {
            vec!["Viewer".to_string()]
        } else {
            req.role_names.clone()
        };

        for rname in &role_names {
            if let Some(role) = repo.find_role_by_name(Some(org_id), rname).await? {
                repo.assign_role_to_member(membership_id, role.id).await?;
            } else {
                return Err(AppError::NotFound(format!("Role '{}' not found in organization", rname)));
            }
        }

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: Some(org_id),
            actor_id: Some(actor_id),
            action: "organization.member_add".to_string(),
            resource: format!("membership:{}", membership_id),
            details: Some(serde_json::json!({ "user_id": user.id, "roles": role_names })),
            ip_address: ip,
            created_at: now,
        }).await;

        Ok(MemberDetail {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            membership_status: "active".to_string(),
            roles: role_names,
            joined_at: now,
        })
    }

    pub async fn list_members(repo: Arc<dyn AppRepository>, org_id: Uuid) -> Result<Vec<MemberDetail>, AppError> {
        repo.list_org_members(org_id).await
    }

    pub async fn remove_member(
        repo: Arc<dyn AppRepository>,
        org_id: Uuid,
        target_user_id: Uuid,
        actor_id: Uuid,
        ip: Option<String>,
    ) -> Result<(), AppError> {
        repo.remove_membership(target_user_id, org_id).await?;

        let _ = repo.log_audit(AuditLog {
            id: Uuid::new_v4(),
            organization_id: Some(org_id),
            actor_id: Some(actor_id),
            action: "organization.member_remove".to_string(),
            resource: format!("user:{}", target_user_id),
            details: None,
            ip_address: ip,
            created_at: Utc::now(),
        }).await;

        Ok(())
    }
}
