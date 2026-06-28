use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    dto::CreateApiKeyRequest,
    error::AppError,
    handlers::roles::OrgQuery,
    middleware::{auth::AuthUser, require_perm::require_permission},
    services::api_key_service::ApiKeyService,
    state::AppState,
};

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&*state.repo, auth.id, payload.organization_id, "api_key:create").await?;
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let created = ApiKeyService::create(state.repo, payload, auth.id, ip).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<OrgQuery>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = query.organization_id
        .ok_or_else(|| AppError::BadRequest("organization_id query param is required".to_string()))?;
    require_permission(&*state.repo, auth.id, org_id, "api_key:read").await?;
    let keys = ApiKeyService::list(state.repo, org_id).await?;
    Ok((StatusCode::OK, Json(keys)))
}

pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<OrgQuery>,
    addr: Option<ConnectInfo<SocketAddr>>,
) -> Result<impl IntoResponse, AppError> {
    let org_id = query.organization_id
        .ok_or_else(|| AppError::BadRequest("organization_id query param is required".to_string()))?;
    require_permission(&*state.repo, auth.id, org_id, "api_key:delete").await?;
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    ApiKeyService::revoke(state.repo, id, org_id, auth.id, ip).await?;
    Ok(StatusCode::NO_CONTENT)
}
