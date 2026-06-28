use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    dto::{AddMemberRequest, CreateOrgRequest, UpdateOrgRequest},
    error::AppError,
    middleware::{auth::AuthUser, require_perm::require_permission},
    models::PaginationQuery,
    services::org_service::OrgService,
    state::AppState,
};

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<CreateOrgRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let org = OrgService::create_org(state.repo, auth.id, payload, ip).await?;
    Ok((StatusCode::CREATED, Json(org)))
}

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let orgs = OrgService::list_orgs(state.repo, &query).await?;
    Ok((StatusCode::OK, Json(orgs)))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let org = OrgService::get_org(state.repo, id).await?;
    Ok((StatusCode::OK, Json(org)))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<UpdateOrgRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&*state.repo, auth.id, id, "organization:update").await?;
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let updated = OrgService::update_org(state.repo, id, payload, auth.id, ip).await?;
    Ok((StatusCode::OK, Json(updated)))
}

pub async fn list_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&*state.repo, auth.id, id, "user:read").await?;
    let members = OrgService::list_members(state.repo, id).await?;
    Ok((StatusCode::OK, Json(members)))
}

pub async fn add_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&*state.repo, auth.id, id, "organization:invite").await?;
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let member = OrgService::add_member(state.repo, id, payload, auth.id, ip).await?;
    Ok((StatusCode::CREATED, Json(member)))
}

pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    addr: Option<ConnectInfo<SocketAddr>>,
) -> Result<impl IntoResponse, AppError> {
    require_permission(&*state.repo, auth.id, org_id, "user:delete").await?;
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    OrgService::remove_member(state.repo, org_id, target_user_id, auth.id, ip).await?;
    Ok(StatusCode::NO_CONTENT)
}
