use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{AuditLog, PaginatedResponse, PaginationQuery},
    repositories::AppRepository,
};

pub struct AuditService;

impl AuditService {
    pub async fn list(
        repo: Arc<dyn AppRepository>,
        org_id: Option<Uuid>,
        query: &PaginationQuery,
    ) -> Result<PaginatedResponse<AuditLog>, AppError> {
        repo.list_audit_logs(org_id, query).await
    }
}
