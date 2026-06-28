use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    dto::{CreatePermissionRequest, EvaluatePermissionRequest},
    error::AppError,
    middleware::auth::AuthUser,
    services::rbac_service::RbacService,
    state::AppState,
};

pub async fn create(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(payload): Json<CreatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let perm = RbacService::create_permission(state.repo, payload).await?;
    Ok((StatusCode::CREATED, Json(perm)))
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let perms = RbacService::list_permissions(state.repo).await?;
    Ok((StatusCode::OK, Json(perms)))
}

pub async fn evaluate(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(payload): Json<EvaluatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = RbacService::evaluate(state.repo, payload.user_id, payload.organization_id, &payload.permission).await?;
    Ok((StatusCode::OK, Json(res)))
}
