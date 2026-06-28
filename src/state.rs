use std::sync::Arc;
use crate::{config::Config, repositories::AppRepository};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub repo: Arc<dyn AppRepository>,
}
