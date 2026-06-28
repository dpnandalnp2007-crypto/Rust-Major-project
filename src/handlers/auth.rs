use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;

use crate::{
    dto::{LoginRequest, RefreshTokenRequest, RegisterRequest},
    error::AppError,
    services::auth_service::AuthService,
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let res = AuthService::register(state.repo, &state.config, payload, ip).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn login(
    State(state): State<AppState>,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = addr.map(|ConnectInfo(a)| a.ip().to_string());
    let res = AuthService::login(state.repo, &state.config, payload, ip).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = AuthService::refresh(state.repo, &state.config, payload).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    AuthService::logout(state.repo, &state.config, payload.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}
