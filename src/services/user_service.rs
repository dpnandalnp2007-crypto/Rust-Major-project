use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dto::UpdateProfileRequest,
    error::AppError,
    models::UserResponse,
    repositories::AppRepository,
};

pub struct UserService;

impl UserService {
    pub async fn get_me(repo: Arc<dyn AppRepository>, user_id: Uuid) -> Result<UserResponse, AppError> {
        let user = repo
            .find_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User profile not found".to_string()))?;
        Ok(user.into())
    }

    pub async fn update_me(
        repo: Arc<dyn AppRepository>,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<UserResponse, AppError> {
        if let Some(full_name) = req.full_name {
            if full_name.trim().is_empty() {
                return Err(AppError::Validation("Full name cannot be empty".to_string()));
            }
            let updated = repo.update_user_full_name(user_id, &full_name).await?;
            return Ok(updated.into());
        }

        Self::get_me(repo, user_id).await
    }
}
