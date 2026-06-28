use std::collections::{HashMap, HashSet};
use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        ApiKey, AuditLog, MemberDetail, Membership, Organization, PaginatedResponse,
        PaginationQuery, Permission, Role, RoleWithPermissions, Session, User,
    },
    repositories::AppRepository,
};

#[derive(Debug, Default)]
pub struct MemoryStore {
    pub users: HashMap<Uuid, User>,
    pub orgs: HashMap<Uuid, Organization>,
    pub memberships: HashMap<Uuid, Membership>,
    pub roles: HashMap<Uuid, Role>,
    pub permissions: HashMap<Uuid, Permission>,
    pub role_perms: HashSet<(Uuid, Uuid)>, // (role_id, perm_id)
    pub member_roles: HashSet<(Uuid, Uuid)>, // (membership_id, role_id)
    pub sessions: HashMap<Uuid, Session>,
    pub api_keys: HashMap<Uuid, ApiKey>,
    pub audit_logs: Vec<AuditLog>,
}

pub struct MemoryRepository {
    store: RwLock<MemoryStore>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        let mut store = MemoryStore::default();
        
        // Seed core permissions
        let perm_names = [
            ("user:create", "Create new users inside organization"),
            ("user:read", "View user profiles and members"),
            ("user:update", "Update user member details"),
            ("user:delete", "Remove users from organization"),
            ("organization:create", "Create new organizations"),
            ("organization:read", "View organization details"),
            ("organization:update", "Update organization settings"),
            ("organization:delete", "Delete organization"),
            ("organization:invite", "Invite members to organization"),
            ("role:create", "Create custom organization roles"),
            ("role:read", "View roles and permission assignments"),
            ("role:update", "Modify custom roles"),
            ("role:delete", "Delete custom roles"),
            ("permission:create", "Create new custom permissions"),
            ("permission:read", "View available permissions"),
            ("permission:assign", "Assign permissions to roles"),
            ("api_key:create", "Generate API keys"),
            ("api_key:read", "View API keys metadata"),
            ("api_key:delete", "Revoke API keys"),
            ("session:read", "View active sessions"),
            ("session:revoke", "Revoke active sessions"),
            ("audit:read", "View audit trail logs"),
        ];

        let mut perm_map = HashMap::new();
        for (name, desc) in perm_names {
            let id = Uuid::new_v4();
            let perm = Permission {
                id,
                name: name.to_string(),
                description: Some(desc.to_string()),
                created_at: Utc::now(),
            };
            store.permissions.insert(id, perm);
            perm_map.insert(name.to_string(), id);
        }

        // Seed global system roles
        let owner_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let admin_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let manager_id = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        let viewer_id = Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();

        store.roles.insert(owner_id, Role {
            id: owner_id,
            organization_id: None,
            name: "Owner".to_string(),
            description: Some("Full administrative access".to_string()),
            is_system: true,
            created_at: Utc::now(),
        });

        store.roles.insert(admin_id, Role {
            id: admin_id,
            organization_id: None,
            name: "Admin".to_string(),
            description: Some("Administrative access excluding org deletion".to_string()),
            is_system: true,
            created_at: Utc::now(),
        });

        store.roles.insert(manager_id, Role {
            id: manager_id,
            organization_id: None,
            name: "Manager".to_string(),
            description: Some("Can manage users and API keys".to_string()),
            is_system: true,
            created_at: Utc::now(),
        });

        store.roles.insert(viewer_id, Role {
            id: viewer_id,
            organization_id: None,
            name: "Viewer".to_string(),
            description: Some("Read-only access".to_string()),
            is_system: true,
            created_at: Utc::now(),
        });

        // Assign all perms to owner
        for perm_id in perm_map.values() {
            store.role_perms.insert((owner_id, *perm_id));
        }

        // Assign admin perms
        for (name, perm_id) in &perm_map {
            if name != "organization:delete" {
                store.role_perms.insert((admin_id, *perm_id));
            }
        }

        Self {
            store: RwLock::new(store),
        }
    }
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AppRepository for MemoryRepository {
    async fn create_user(&self, user: User) -> Result<User, AppError> {
        let mut store = self.store.write().await;
        if store.users.values().any(|u| u.email.eq_ignore_ascii_case(&user.email)) {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }
        store.users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let store = self.store.read().await;
        Ok(store.users.values().find(|u| u.email.eq_ignore_ascii_case(email)).cloned())
    }

    async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let store = self.store.read().await;
        Ok(store.users.get(&id).cloned())
    }

    async fn update_user_full_name(&self, id: Uuid, full_name: &str) -> Result<User, AppError> {
        let mut store = self.store.write().await;
        let user = store.users.get_mut(&id).ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        user.full_name = full_name.to_string();
        user.updated_at = Utc::now();
        Ok(user.clone())
    }

    async fn create_org(&self, org: Organization) -> Result<Organization, AppError> {
        let mut store = self.store.write().await;
        if store.orgs.values().any(|o| o.slug.eq_ignore_ascii_case(&org.slug)) {
            return Err(AppError::Conflict("Organization slug already exists".to_string()));
        }
        store.orgs.insert(org.id, org.clone());
        Ok(org)
    }

    async fn find_org_by_id(&self, id: Uuid) -> Result<Option<Organization>, AppError> {
        let store = self.store.read().await;
        Ok(store.orgs.get(&id).cloned())
    }

    async fn find_org_by_slug(&self, slug: &str) -> Result<Option<Organization>, AppError> {
        let store = self.store.read().await;
        Ok(store.orgs.values().find(|o| o.slug.eq_ignore_ascii_case(slug)).cloned())
    }

    async fn list_orgs(&self, query: &PaginationQuery) -> Result<PaginatedResponse<Organization>, AppError> {
        let store = self.store.read().await;
        let mut list: Vec<Organization> = store.orgs.values().cloned().collect();
        
        if let Some(search) = &query.search {
            let s = search.to_lowercase();
            list.retain(|o| o.name.to_lowercase().contains(&s) || o.slug.to_lowercase().contains(&s));
        }
        
        list.sort_by_key(|o| o.created_at);
        let total = list.len() as i64;
        
        let offset = query.offset() as usize;
        let limit = query.limit() as usize;
        
        let data = if offset >= list.len() {
            Vec::new()
        } else {
            list[offset..(offset + limit).min(list.len())].to_vec()
        };

        Ok(PaginatedResponse::new(data, total, query))
    }

    async fn update_org(&self, id: Uuid, name: Option<&str>, desc: Option<&str>) -> Result<Organization, AppError> {
        let mut store = self.store.write().await;
        let org = store.orgs.get_mut(&id).ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;
        if let Some(n) = name {
            org.name = n.to_string();
        }
        if let Some(d) = desc {
            org.description = Some(d.to_string());
        }
        org.updated_at = Utc::now();
        Ok(org.clone())
    }

    async fn add_membership(&self, membership: Membership) -> Result<Membership, AppError> {
        let mut store = self.store.write().await;
        if store.memberships.values().any(|m| m.user_id == membership.user_id && m.organization_id == membership.organization_id) {
            return Err(AppError::Conflict("User is already a member of this organization".to_string()));
        }
        store.memberships.insert(membership.id, membership.clone());
        Ok(membership)
    }

    async fn find_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<Option<Membership>, AppError> {
        let store = self.store.read().await;
        Ok(store.memberships.values().find(|m| m.user_id == user_id && m.organization_id == org_id).cloned())
    }

    async fn remove_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        if let Some(id) = store.memberships.values().find(|m| m.user_id == user_id && m.organization_id == org_id).map(|m| m.id) {
            store.memberships.remove(&id);
            store.member_roles.retain(|(mid, _)| *mid != id);
        }
        Ok(())
    }

    async fn list_org_members(&self, org_id: Uuid) -> Result<Vec<MemberDetail>, AppError> {
        let store = self.store.read().await;
        let mut details = Vec::new();
        
        for m in store.memberships.values().filter(|m| m.organization_id == org_id) {
            if let Some(user) = store.users.get(&m.user_id) {
                let role_ids: Vec<Uuid> = store.member_roles.iter().filter(|(mid, _)| *mid == m.id).map(|(_, rid)| *rid).collect();
                let roles: Vec<String> = role_ids.iter().filter_map(|rid| store.roles.get(rid).map(|r| r.name.clone())).collect();
                
                details.push(MemberDetail {
                    user_id: user.id,
                    email: user.email.clone(),
                    full_name: user.full_name.clone(),
                    membership_status: m.status.clone(),
                    roles,
                    joined_at: m.created_at,
                });
            }
        }
        Ok(details)
    }

    async fn create_role(&self, role: Role) -> Result<Role, AppError> {
        let mut store = self.store.write().await;
        if store.roles.values().any(|r| r.organization_id == role.organization_id && r.name.eq_ignore_ascii_case(&role.name)) {
            return Err(AppError::Conflict("Role with this name already exists in organization".to_string()));
        }
        store.roles.insert(role.id, role.clone());
        Ok(role)
    }

    async fn find_role_by_id(&self, id: Uuid) -> Result<Option<Role>, AppError> {
        let store = self.store.read().await;
        Ok(store.roles.get(&id).cloned())
    }

    async fn find_role_by_name(&self, org_id: Option<Uuid>, name: &str) -> Result<Option<Role>, AppError> {
        let store = self.store.read().await;
        Ok(store.roles.values().find(|r| (r.organization_id == org_id || r.organization_id.is_none()) && r.name.eq_ignore_ascii_case(name)).cloned())
    }

    async fn list_roles(&self, org_id: Option<Uuid>) -> Result<Vec<RoleWithPermissions>, AppError> {
        let store = self.store.read().await;
        let mut res = Vec::new();
        
        for r in store.roles.values().filter(|r| r.organization_id == org_id || r.organization_id.is_none()) {
            let perm_ids: Vec<Uuid> = store.role_perms.iter().filter(|(rid, _)| *rid == r.id).map(|(_, pid)| *pid).collect();
            let permissions: Vec<String> = perm_ids.iter().filter_map(|pid| store.permissions.get(pid).map(|p| p.name.clone())).collect();
            
            res.push(RoleWithPermissions {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name.clone(),
                description: r.description.clone(),
                is_system: r.is_system,
                permissions,
                created_at: r.created_at,
            });
        }
        Ok(res)
    }

    async fn update_role(&self, id: Uuid, name: Option<&str>, desc: Option<&str>) -> Result<Role, AppError> {
        let mut store = self.store.write().await;
        let role = store.roles.get_mut(&id).ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;
        if role.is_system {
            return Err(AppError::Forbidden("Cannot modify default system roles".to_string()));
        }
        if let Some(n) = name {
            role.name = n.to_string();
        }
        if let Some(d) = desc {
            role.description = Some(d.to_string());
        }
        Ok(role.clone())
    }

    async fn delete_role(&self, id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        if let Some(r) = store.roles.get(&id) {
            if r.is_system {
                return Err(AppError::Forbidden("Cannot delete system roles".to_string()));
            }
        }
        store.roles.remove(&id);
        store.role_perms.retain(|(rid, _)| *rid != id);
        store.member_roles.retain(|(_, rid)| *rid != id);
        Ok(())
    }

    async fn assign_permission_to_role(&self, role_id: Uuid, perm_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        store.role_perms.insert((role_id, perm_id));
        Ok(())
    }

    async fn remove_permissions_from_role(&self, role_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        store.role_perms.retain(|(rid, _)| *rid != role_id);
        Ok(())
    }

    async fn create_permission(&self, perm: Permission) -> Result<Permission, AppError> {
        let mut store = self.store.write().await;
        if store.permissions.values().any(|p| p.name.eq_ignore_ascii_case(&perm.name)) {
            return Err(AppError::Conflict("Permission name already exists".to_string()));
        }
        store.permissions.insert(perm.id, perm.clone());
        Ok(perm)
    }

    async fn list_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let store = self.store.read().await;
        Ok(store.permissions.values().cloned().collect())
    }

    async fn find_permission_by_name(&self, name: &str) -> Result<Option<Permission>, AppError> {
        let store = self.store.read().await;
        Ok(store.permissions.values().find(|p| p.name.eq_ignore_ascii_case(name)).cloned())
    }

    async fn assign_role_to_member(&self, membership_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        store.member_roles.insert((membership_id, role_id));
        Ok(())
    }

    async fn remove_roles_from_member(&self, membership_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        store.member_roles.retain(|(mid, _)| *mid != membership_id);
        Ok(())
    }

    async fn has_permission(&self, user_id: Uuid, org_id: Uuid, permission_name: &str) -> Result<bool, AppError> {
        let store = self.store.read().await;
        
        let membership = match store.memberships.values().find(|m| m.user_id == user_id && m.organization_id == org_id) {
            Some(m) => m,
            None => return Ok(false),
        };

        if membership.status != "active" {
            return Ok(false);
        }

        let perm_id = match store.permissions.values().find(|p| p.name.eq_ignore_ascii_case(permission_name)).map(|p| p.id) {
            Some(pid) => pid,
            None => return Ok(false),
        };

        let role_ids: Vec<Uuid> = store.member_roles.iter().filter(|(mid, _)| *mid == membership.id).map(|(_, rid)| *rid).collect();
        
        for rid in role_ids {
            if store.role_perms.contains(&(rid, perm_id)) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    async fn create_session(&self, session: Session) -> Result<Session, AppError> {
        let mut store = self.store.write().await;
        store.sessions.insert(session.id, session.clone());
        Ok(session)
    }

    async fn find_session_by_hash(&self, hash: &str) -> Result<Option<Session>, AppError> {
        let store = self.store.read().await;
        Ok(store.sessions.values().find(|s| s.refresh_token_hash == hash).cloned())
    }

    async fn list_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>, AppError> {
        let store = self.store.read().await;
        Ok(store.sessions.values().filter(|s| s.user_id == user_id && s.expires_at > Utc::now()).cloned().collect())
    }

    async fn revoke_session(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        if let Some(s) = store.sessions.get(&id) {
            if s.user_id != user_id {
                return Err(AppError::Forbidden("Cannot revoke another user's session".to_string()));
            }
        }
        store.sessions.remove(&id);
        Ok(())
    }

    async fn create_api_key(&self, api_key: ApiKey) -> Result<ApiKey, AppError> {
        let mut store = self.store.write().await;
        store.api_keys.insert(api_key.id, api_key.clone());
        Ok(api_key)
    }

    async fn find_api_key_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, AppError> {
        let store = self.store.read().await;
        Ok(store.api_keys.values().find(|k| k.key_hash == hash && k.status == "active").cloned())
    }

    async fn list_org_api_keys(&self, org_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        let store = self.store.read().await;
        Ok(store.api_keys.values().filter(|k| k.organization_id == org_id).cloned().collect())
    }

    async fn revoke_api_key(&self, id: Uuid, org_id: Uuid) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        if let Some(k) = store.api_keys.get_mut(&id) {
            if k.organization_id != org_id {
                return Err(AppError::Forbidden("Cannot revoke API key from another organization".to_string()));
            }
            k.status = "revoked".to_string();
        }
        Ok(())
    }

    async fn log_audit(&self, audit: AuditLog) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        store.audit_logs.push(audit);
        Ok(())
    }

    async fn list_audit_logs(&self, org_id: Option<Uuid>, query: &PaginationQuery) -> Result<PaginatedResponse<AuditLog>, AppError> {
        let store = self.store.read().await;
        let mut list: Vec<AuditLog> = store.audit_logs.iter().filter(|a| org_id.is_none() || a.organization_id == org_id).cloned().collect();
        
        list.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        let total = list.len() as i64;
        
        let offset = query.offset() as usize;
        let limit = query.limit() as usize;
        
        let data = if offset >= list.len() {
            Vec::new()
        } else {
            list[offset..(offset + limit).min(list.len())].to_vec()
        };

        Ok(PaginatedResponse::new(data, total, query))
    }
}
