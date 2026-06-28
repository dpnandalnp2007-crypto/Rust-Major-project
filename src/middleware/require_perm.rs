use uuid::Uuid;
use crate::{error::AppError, repositories::AppRepository};

pub async fn require_permission(
    repo: &dyn AppRepository,
    user_id: Uuid,
    org_id: Uuid,
    permission: &str,
) -> Result<(), AppError> {
    let has = repo.has_permission(user_id, org_id, permission).await?;
    if !has {
        return Err(AppError::Forbidden(format!(
            "Missing required permission '{}' in organization '{}'",
            permission, org_id
        )));
    }
    Ok(())
}
