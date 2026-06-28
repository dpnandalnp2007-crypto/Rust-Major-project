use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub permission_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatePermissionRequest {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatePermissionResponse {
    pub allowed: bool,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub permission: String,
}
