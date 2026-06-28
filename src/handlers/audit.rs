use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{auth::AuthUser, require_perm::require_permission},
    models::PaginationQuery,
    services::audit_service::AuditService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub organization_id: Option<Uuid>,
    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<AuditQuery>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(org_id) = query.organization_id {
        require_permission(&*state.repo, auth.id, org_id, "audit:read").await?;
    }
    let logs = AuditService::list(state.repo, query.organization_id, &query.pagination).await?;
    Ok((StatusCode::OK, Json(logs)))
}
