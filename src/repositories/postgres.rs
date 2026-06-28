use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        ApiKey, AuditLog, MemberDetail, Membership, Organization, PaginatedResponse,
        PaginationQuery, Permission, Role, RoleWithPermissions, Session, User,
    },
    repositories::AppRepository,
};

#[derive(Clone)]
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppRepository for PostgresRepository {
    async fn create_user(&self, user: User) -> Result<User, AppError> {
        let rec = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (id, email, password_hash, full_name, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, email, password_hash, full_name, status, created_at, updated_at"#
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.full_name)
        .bind(&user.status)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::Conflict("User email already exists".to_string())
            }
            _ => AppError::from(e)
        })?;

        Ok(rec)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let rec = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, full_name, status, created_at, updated_at FROM users WHERE LOWER(email) = LOWER($1)"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let rec = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, full_name, status, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn update_user_full_name(&self, id: Uuid, full_name: &str) -> Result<User, AppError> {
        let rec = sqlx::query_as::<_, User>(
            r#"UPDATE users SET full_name = $2, updated_at = CURRENT_TIMESTAMP
               WHERE id = $1
               RETURNING id, email, password_hash, full_name, status, created_at, updated_at"#
        )
        .bind(id)
        .bind(full_name)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn create_org(&self, org: Organization) -> Result<Organization, AppError> {
        let rec = sqlx::query_as::<_, Organization>(
            r#"INSERT INTO organizations (id, name, slug, description, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, name, slug, description, created_at, updated_at"#
        )
        .bind(org.id)
        .bind(&org.name)
        .bind(&org.slug)
        .bind(&org.description)
        .bind(org.created_at)
        .bind(org.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::Conflict("Organization slug already exists".to_string())
            }
            _ => AppError::from(e)
        })?;
        Ok(rec)
    }

    async fn find_org_by_id(&self, id: Uuid) -> Result<Option<Organization>, AppError> {
        let rec = sqlx::query_as::<_, Organization>(
            "SELECT id, name, slug, description, created_at, updated_at FROM organizations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_org_by_slug(&self, slug: &str) -> Result<Option<Organization>, AppError> {
        let rec = sqlx::query_as::<_, Organization>(
            "SELECT id, name, slug, description, created_at, updated_at FROM organizations WHERE LOWER(slug) = LOWER($1)"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn list_orgs(&self, query: &PaginationQuery) -> Result<PaginatedResponse<Organization>, AppError> {
        let limit = query.limit();
        let offset = query.offset();
        let search = query.search.as_deref().unwrap_or("");

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM organizations WHERE $1 = '' OR name ILIKE '%' || $1 || '%' OR slug ILIKE '%' || $1 || '%'"
        )
        .bind(search)
        .fetch_one(&self.pool)
        .await?;

        let data = sqlx::query_as::<_, Organization>(
            r#"SELECT id, name, slug, description, created_at, updated_at FROM organizations
               WHERE $1 = '' OR name ILIKE '%' || $1 || '%' OR slug ILIKE '%' || $1 || '%'
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        )
        .bind(search)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResponse::new(data, total, query))
    }

    async fn update_org(&self, id: Uuid, name: Option<&str>, desc: Option<&str>) -> Result<Organization, AppError> {
        let rec = sqlx::query_as::<_, Organization>(
            r#"UPDATE organizations
               SET name = COALESCE($2, name),
                   description = COALESCE($3, description),
                   updated_at = CURRENT_TIMESTAMP
               WHERE id = $1
               RETURNING id, name, slug, description, created_at, updated_at"#
        )
        .bind(id)
        .bind(name)
        .bind(desc)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn add_membership(&self, membership: Membership) -> Result<Membership, AppError> {
        let rec = sqlx::query_as::<_, Membership>(
            r#"INSERT INTO memberships (id, user_id, organization_id, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, user_id, organization_id, status, created_at, updated_at"#
        )
        .bind(membership.id)
        .bind(membership.user_id)
        .bind(membership.organization_id)
        .bind(&membership.status)
        .bind(membership.created_at)
        .bind(membership.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::Conflict("User is already a member of this organization".to_string())
            }
            _ => AppError::from(e)
        })?;
        Ok(rec)
    }

    async fn find_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<Option<Membership>, AppError> {
        let rec = sqlx::query_as::<_, Membership>(
            "SELECT id, user_id, organization_id, status, created_at, updated_at FROM memberships WHERE user_id = $1 AND organization_id = $2"
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn remove_membership(&self, user_id: Uuid, org_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM memberships WHERE user_id = $1 AND organization_id = $2")
            .bind(user_id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_org_members(&self, org_id: Uuid) -> Result<Vec<MemberDetail>, AppError> {
        let rows = sqlx::query(
            r#"SELECT u.id as user_id, u.email, u.full_name, m.status as membership_status, m.created_at as joined_at,
                      COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), '{}') as roles
               FROM memberships m
               JOIN users u ON m.user_id = u.id
               LEFT JOIN member_roles mr ON m.id = mr.membership_id
               LEFT JOIN roles r ON mr.role_id = r.id
               WHERE m.organization_id = $1
               GROUP BY u.id, u.email, u.full_name, m.status, m.created_at
               ORDER BY m.created_at ASC"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        let mut res = Vec::new();
        for row in rows {
            res.push(MemberDetail {
                user_id: row.get("user_id"),
                email: row.get("email"),
                full_name: row.get("full_name"),
                membership_status: row.get("membership_status"),
                roles: row.get("roles"),
                joined_at: row.get("joined_at"),
            });
        }
        Ok(res)
    }

    async fn create_role(&self, role: Role) -> Result<Role, AppError> {
        let rec = sqlx::query_as::<_, Role>(
            r#"INSERT INTO roles (id, organization_id, name, description, is_system, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, organization_id, name, description, is_system, created_at"#
        )
        .bind(role.id)
        .bind(role.organization_id)
        .bind(&role.name)
        .bind(&role.description)
        .bind(role.is_system)
        .bind(role.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::Conflict("Role name already exists in this organization".to_string())
            }
            _ => AppError::from(e)
        })?;
        Ok(rec)
    }

    async fn find_role_by_id(&self, id: Uuid) -> Result<Option<Role>, AppError> {
        let rec = sqlx::query_as::<_, Role>(
            "SELECT id, organization_id, name, description, is_system, created_at FROM roles WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_role_by_name(&self, org_id: Option<Uuid>, name: &str) -> Result<Option<Role>, AppError> {
        let query = match org_id {
            Some(oid) => sqlx::query_as::<_, Role>(
                "SELECT id, organization_id, name, description, is_system, created_at FROM roles WHERE (organization_id = $1 OR organization_id IS NULL) AND LOWER(name) = LOWER($2) ORDER BY organization_id NULLS LAST LIMIT 1"
            ).bind(oid).bind(name),
            None => sqlx::query_as::<_, Role>(
                "SELECT id, organization_id, name, description, is_system, created_at FROM roles WHERE organization_id IS NULL AND LOWER(name) = LOWER($1) LIMIT 1"
            ).bind(name),
        };
        let rec = query.fetch_optional(&self.pool).await?;
        Ok(rec)
    }

    async fn list_roles(&self, org_id: Option<Uuid>) -> Result<Vec<RoleWithPermissions>, AppError> {
        let rows = sqlx::query(
            r#"SELECT r.id, r.organization_id, r.name, r.description, r.is_system, r.created_at,
                      COALESCE(array_agg(p.name) FILTER (WHERE p.name IS NOT NULL), '{}') as permissions
               FROM roles r
               LEFT JOIN role_permissions rp ON r.id = rp.role_id
               LEFT JOIN permissions p ON rp.permission_id = p.id
               WHERE r.organization_id IS NULL OR r.organization_id = $1
               GROUP BY r.id, r.organization_id, r.name, r.description, r.is_system, r.created_at
               ORDER BY r.is_system DESC, r.name ASC"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        let mut res = Vec::new();
        for row in rows {
            res.push(RoleWithPermissions {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                name: row.get("name"),
                description: row.get("description"),
                is_system: row.get("is_system"),
                permissions: row.get("permissions"),
                created_at: row.get("created_at"),
            });
        }
        Ok(res)
    }

    async fn update_role(&self, id: Uuid, name: Option<&str>, desc: Option<&str>) -> Result<Role, AppError> {
        let rec = sqlx::query_as::<_, Role>(
            r#"UPDATE roles
               SET name = COALESCE($2, name), description = COALESCE($3, description)
               WHERE id = $1 AND is_system = false
               RETURNING id, organization_id, name, description, is_system, created_at"#
        )
        .bind(id)
        .bind(name)
        .bind(desc)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::Forbidden("Role not found or is a protected system role".to_string()))?;
        Ok(rec)
    }

    async fn delete_role(&self, id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM roles WHERE id = $1 AND is_system = false")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::Forbidden("Role not found or is a protected system role".to_string()));
        }
        Ok(())
    }

    async fn assign_permission_to_role(&self, role_id: Uuid, perm_id: Uuid) -> Result<(), AppError> {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(role_id)
            .bind(perm_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn remove_permissions_from_role(&self, role_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_permission(&self, perm: Permission) -> Result<Permission, AppError> {
        let rec = sqlx::query_as::<_, Permission>(
            r#"INSERT INTO permissions (id, name, description, created_at)
               VALUES ($1, $2, $3, $4)
               RETURNING id, name, description, created_at"#
        )
        .bind(perm.id)
        .bind(&perm.name)
        .bind(&perm.description)
        .bind(perm.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::Conflict("Permission name already exists".to_string())
            }
            _ => AppError::from(e)
        })?;
        Ok(rec)
    }

    async fn list_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let recs = sqlx::query_as::<_, Permission>("SELECT id, name, description, created_at FROM permissions ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await?;
        Ok(recs)
    }

    async fn find_permission_by_name(&self, name: &str) -> Result<Option<Permission>, AppError> {
        let rec = sqlx::query_as::<_, Permission>("SELECT id, name, description, created_at FROM permissions WHERE LOWER(name) = LOWER($1)")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(rec)
    }

    async fn assign_role_to_member(&self, membership_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        sqlx::query("INSERT INTO member_roles (membership_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(membership_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn remove_roles_from_member(&self, membership_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM member_roles WHERE membership_id = $1")
            .bind(membership_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn has_permission(&self, user_id: Uuid, org_id: Uuid, permission_name: &str) -> Result<bool, AppError> {
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS (
                 SELECT 1 FROM memberships m
                 JOIN member_roles mr ON m.id = mr.membership_id
                 JOIN role_permissions rp ON mr.role_id = rp.role_id
                 JOIN permissions p ON rp.permission_id = p.id
                 WHERE m.user_id = $1 AND m.organization_id = $2 AND m.status = 'active'
                   AND LOWER(p.name) = LOWER($3)
               )"#
        )
        .bind(user_id)
        .bind(org_id)
        .bind(permission_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn create_session(&self, session: Session) -> Result<Session, AppError> {
        let rec = sqlx::query_as::<_, Session>(
            r#"INSERT INTO sessions (id, user_id, refresh_token_hash, device_info, ip_address, expires_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, user_id, refresh_token_hash, device_info, ip_address, expires_at, created_at"#
        )
        .bind(session.id)
        .bind(session.user_id)
        .bind(&session.refresh_token_hash)
        .bind(&session.device_info)
        .bind(&session.ip_address)
        .bind(session.expires_at)
        .bind(session.created_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_session_by_hash(&self, hash: &str) -> Result<Option<Session>, AppError> {
        let rec = sqlx::query_as::<_, Session>(
            "SELECT id, user_id, refresh_token_hash, device_info, ip_address, expires_at, created_at FROM sessions WHERE refresh_token_hash = $1 AND expires_at > CURRENT_TIMESTAMP"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn list_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>, AppError> {
        let recs = sqlx::query_as::<_, Session>(
            "SELECT id, user_id, refresh_token_hash, device_info, ip_address, expires_at, created_at FROM sessions WHERE user_id = $1 AND expires_at > CURRENT_TIMESTAMP ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    async fn revoke_session(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Session not found or already revoked".to_string()));
        }
        Ok(())
    }

    async fn create_api_key(&self, api_key: ApiKey) -> Result<ApiKey, AppError> {
        let rec = sqlx::query_as::<_, ApiKey>(
            r#"INSERT INTO api_keys (id, organization_id, name, description, key_prefix, key_hash, scopes, status, expires_at, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING id, organization_id, name, description, key_prefix, key_hash, scopes, status, expires_at, created_at"#
        )
        .bind(api_key.id)
        .bind(api_key.organization_id)
        .bind(&api_key.name)
        .bind(&api_key.description)
        .bind(&api_key.key_prefix)
        .bind(&api_key.key_hash)
        .bind(&api_key.scopes)
        .bind(&api_key.status)
        .bind(api_key.expires_at)
        .bind(api_key.created_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_api_key_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, AppError> {
        let rec = sqlx::query_as::<_, ApiKey>(
            "SELECT id, organization_id, name, description, key_prefix, key_hash, scopes, status, expires_at, created_at FROM api_keys WHERE key_hash = $1 AND status = 'active' AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn list_org_api_keys(&self, org_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        let recs = sqlx::query_as::<_, ApiKey>(
            "SELECT id, organization_id, name, description, key_prefix, key_hash, scopes, status, expires_at, created_at FROM api_keys WHERE organization_id = $1 ORDER BY created_at DESC"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    async fn revoke_api_key(&self, id: Uuid, org_id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("UPDATE api_keys SET status = 'revoked' WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("API key not found".to_string()));
        }
        Ok(())
    }

    async fn log_audit(&self, audit: AuditLog) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO audit_logs (id, organization_id, actor_id, action, resource, details, ip_address, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#
        )
        .bind(audit.id)
        .bind(audit.organization_id)
        .bind(audit.actor_id)
        .bind(&audit.action)
        .bind(&audit.resource)
        .bind(&audit.details)
        .bind(&audit.ip_address)
        .bind(audit.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_audit_logs(&self, org_id: Option<Uuid>, query: &PaginationQuery) -> Result<PaginatedResponse<AuditLog>, AppError> {
        let limit = query.limit();
        let offset = query.offset();

        let total: i64 = match org_id {
            Some(oid) => sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE organization_id = $1").bind(oid).fetch_one(&self.pool).await?,
            None => sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs").fetch_one(&self.pool).await?,
        };

        let data = match org_id {
            Some(oid) => sqlx::query_as::<_, AuditLog>(
                "SELECT id, organization_id, actor_id, action, resource, details, ip_address, created_at FROM audit_logs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            ).bind(oid).bind(limit).bind(offset).fetch_all(&self.pool).await?,
            None => sqlx::query_as::<_, AuditLog>(
                "SELECT id, organization_id, actor_id, action, resource, details, ip_address, created_at FROM audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            ).bind(limit).bind(offset).fetch_all(&self.pool).await?,
        };

        Ok(PaginatedResponse::new(data, total, query))
    }
}
