use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::SessionResponse,
    repositories::AppRepository,
};

pub struct SessionService;

impl SessionService {
    pub async fn list_sessions(
        repo: Arc<dyn AppRepository>,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<Vec<SessionResponse>, AppError> {
        let sessions = repo.list_user_sessions(user_id).await?;
        
        let res = sessions
            .into_iter()
            .map(|s| SessionResponse {
                id: s.id,
                user_id: s.user_id,
                device_info: s.device_info,
                ip_address: s.ip_address,
                expires_at: s.expires_at,
                created_at: s.created_at,
                is_current: s.id == current_session_id,
            })
            .collect();

        Ok(res)
    }

    pub async fn revoke(repo: Arc<dyn AppRepository>, session_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        repo.revoke_session(session_id, user_id).await
    }
}
