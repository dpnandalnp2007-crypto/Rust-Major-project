use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use iam_platform::{
    config::Config,
    repositories::{AppRepository, MemoryRepository, PostgresRepository},
    routes::create_router,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "iam_platform=debug,tower_http=debug,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Identity & Access Management (IAM) Platform...");

    let config = Config::from_env();
    tracing::info!("Loaded configuration. Target host: {}:{}", config.host, config.port);

    // Initialize repository store
    let repo: Arc<dyn AppRepository> = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("Connected to PostgreSQL database at {}", config.database_url);
            match sqlx::migrate!("./migrations").run(&pool).await {
                Ok(_) => tracing::info!("Successfully ran database migrations"),
                Err(e) => tracing::warn!("Migration execution warning: {}", e),
            }
            Arc::new(PostgresRepository::new(pool))
        }
        Err(e) => {
            tracing::warn!("Could not connect to PostgreSQL ({}). Initializing high-performance in-memory repository store.", e);
            Arc::new(MemoryRepository::new())
        }
    };

    let state = AppState {
        config: config.clone(),
        repo,
    };

    let app = create_router(state);

    let listener_addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&listener_addr).await?;
    tracing::info!("IAM Platform listening on http://{}", listener_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
