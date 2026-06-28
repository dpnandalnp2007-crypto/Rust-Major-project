use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    dto::{CreateRoleRequest, UpdateRoleRequest},
    error::AppError,
    middleware::{auth::AuthUser, require_perm::require_permission},
    services::rbac_service::RbacService,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct OrgQuery {
    pub organization_id: Option<Uuid>,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(org_id) = payload.organization_id {
        require_permission(&*state.repo, auth.id, org_id, "role:create").await?;
    } else {
        return Err(AppError::Forbidden("Only system administrators can create global roles".to_string()));
    }

    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let role = RbacService::create_role(state.repo, payload.organization_id, payload, auth.id, ip).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<OrgQuery>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(org_id) = query.organization_id {
        require_permission(&*state.repo, auth.id, org_id, "role:read").await?;
    }
    let roles = RbacService::list_roles(state.repo, query.organization_id).await?;
    Ok((StatusCode::OK, Json(roles)))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let role = state.repo.find_role_by_id(id).await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

    if let Some(org_id) = role.organization_id {
        require_permission(&*state.repo, auth.id, org_id, "role:update").await?;
    } else {
        return Err(AppError::Forbidden("Cannot modify system default roles".to_string()));
    }

    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let updated = RbacService::update_role(state.repo, id, payload, auth.id, ip).await?;
    Ok((StatusCode::OK, Json(updated)))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    addr: Option<ConnectInfo<SocketAddr>>,
) -> Result<impl IntoResponse, AppError> {
    let role = state.repo.find_role_by_id(id).await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

    if let Some(org_id) = role.organization_id {
        require_permission(&*state.repo, auth.id, org_id, "role:delete").await?;
    } else {
        return Err(AppError::Forbidden("Cannot delete system default roles".to_string()));
    }

    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    RbacService::delete_role(state.repo, id, auth.id, ip).await?;
    Ok(StatusCode::NO_CONTENT)
}
