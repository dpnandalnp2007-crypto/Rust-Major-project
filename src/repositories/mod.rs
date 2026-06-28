pub mod memory;
pub mod postgres;

pub use memory::MemoryRepository;
pub use postgres::PostgresRepository;

use async_trait::async_trait;
use uuid::Uuid;
use crate::{
    error::AppError,
    models::{
        ApiKey, AuditLog, MemberDetail, Membership, Organization, PaginatedResponse,
        PaginationQuery, Permission, Role, RoleWithPermissions, Session, User,
    },
};

#[async_trait]
pub trait AppRepository: Send + Sync {
    // --- User Operations ---
    async fn create_user(&self, user: User) -> Result<User, AppError>;
    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn update_user_full_name(&self, id: Uuid, full_name: &str) -> Result<User, AppError>;

    // --- Organization Operations ---
    async fn create_org(&self, org: Organization) -> Result<Organization, AppError>;
    async fn find_org_by_id(&self, id: Uuid) -> Result<Option<Organization>, AppError>;
    async fn find_org_by_slug(&self, slug: &str) -> Result<Option<Organization>, AppError>;
    async fn list_orgs(&self, query: &PaginationQuery) -> Result<PaginatedResponse<Organization>, AppError>;
    async fn update_org(&self, id: Uuid, name: Option<&str>, desc: Option<&str>) -> Result<Organization, AppError>;
    
    // --- Membership Operations ---
    async fn add_membership(&self, membership: Membership) -> Result<Membership, AppError>;
    async fn find_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<Option<Membership>, AppError>;
    async fn remove_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<(), AppError>;
    async fn list_org_members(&self, org_id: Uuid) -> Result<Vec<MemberDetail>, AppError>;

    // --- RBAC Operations ---
    async fn create_role(&self, role: Role) -> Result<Role, AppError>;
    async fn find_role_by_id(&self, id: Uuid) -> Result<Option<Role>, AppError>;
    async fn find_role_by_name(&self, org_id: Option<Uuid>, name: &str) -> Result<Option<Role>, AppError>;
    async fn list_roles(&self, org_id: Option<Uuid>) -> Result<Vec<RoleWithPermissions>, AppError>;
    async fn update_role(&self, id: Uuid, name: Option<&str>, desc: Option<&str>) -> Result<Role, AppError>;
    async fn delete_role(&self, id: Uuid) -> Result<(), AppError>;
    
    async fn assign_permission_to_role(&self, role_id: Uuid, perm_id: Uuid) -> Result<(), AppError>;
    async fn remove_permissions_from_role(&self, role_id: Uuid) -> Result<(), AppError>;
    
    async fn create_permission(&self, perm: Permission) -> Result<Permission, AppError>;
    async fn list_permissions(&self) -> Result<Vec<Permission>, AppError>;
    async fn find_permission_by_name(&self, name: &str) -> Result<Option<Permission>, AppError>;
    
    async fn assign_role_to_member(&self, membership_id: Uuid, role_id: Uuid) -> Result<(), AppError>;
    async fn remove_roles_from_member(&self, membership_id: Uuid) -> Result<(), AppError>;
    
    async fn has_permission(&self, user_id: Uuid, org_id: Uuid, permission_name: &str) -> Result<bool, AppError>;

    // --- Session Operations ---
    async fn create_session(&self, session: Session) -> Result<Session, AppError>;
    async fn find_session_by_hash(&self, hash: &str) -> Result<Option<Session>, AppError>;
    async fn list_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>, AppError>;
    async fn revoke_session(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    // --- API Key Operations ---
    async fn create_api_key(&self, api_key: ApiKey) -> Result<ApiKey, AppError>;
    async fn find_api_key_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, AppError>;
    async fn list_org_api_keys(&self, org_id: Uuid) -> Result<Vec<ApiKey>, AppError>;
    async fn revoke_api_key(&self, id: Uuid, org_id: Uuid) -> Result<(), AppError>;

    // --- Audit Operations ---
    async fn log_audit(&self, audit: AuditLog) -> Result<(), AppError>;
    async fn list_audit_logs(&self, org_id: Option<Uuid>, query: &PaginationQuery) -> Result<PaginatedResponse<AuditLog>, AppError>;
}
