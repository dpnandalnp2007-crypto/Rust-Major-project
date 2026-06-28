use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    dto::UpdateProfileRequest,
    error::AppError,
    middleware::auth::AuthUser,
    services::user_service::UserService,
    state::AppState,
};

pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = UserService::get_me(state.repo, auth.id).await?;
    Ok((StatusCode::OK, Json(user)))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::update_me(state.repo, auth.id, payload).await?;
    Ok((StatusCode::OK, Json(updated)))
}
