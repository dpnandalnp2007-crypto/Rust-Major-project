use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    handlers::{api_keys, audit, auth, organizations, permissions, roles, sessions, users},
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Auth routes
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        
        // User routes
        .route("/users/me", get(users::get_me).patch(users::update_me))
        
        // Organization routes
        .route("/organizations", post(organizations::create).get(organizations::list))
        .route("/organizations/:id", get(organizations::get).patch(organizations::update))
        .route("/organizations/:id/members", get(organizations::list_members).post(organizations::add_member))
        .route("/organizations/:id/members/:user_id", delete(organizations::remove_member))
        
        // Role routes
        .route("/roles", post(roles::create).get(roles::list))
        .route("/roles/:id", patch(roles::update).delete(roles::delete))
        
        // Permission routes
        .route("/permissions", post(permissions::create).get(permissions::list))
        .route("/permissions/evaluate", post(permissions::evaluate))
        
        // Session routes
        .route("/sessions", get(sessions::list))
        .route("/sessions/:id", delete(sessions::revoke))
        
        // API Key routes
        .route("/api-keys", post(api_keys::create).get(api_keys::list))
        .route("/api-keys/:id", delete(api_keys::revoke))
        
        // Audit log routes
        .route("/audit-logs", get(audit::list))
        
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
