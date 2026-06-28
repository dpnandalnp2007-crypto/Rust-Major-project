use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    services::session_service::SessionService,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let dummy_current = Uuid::nil(); // Or extract from context if available
    let sessions = SessionService::list_sessions(state.repo, auth.id, dummy_current).await?;
    Ok((StatusCode::OK, Json(sessions)))
}

pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    SessionService::revoke(state.repo, id, auth.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
