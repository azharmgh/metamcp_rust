//! MetaMCP Server - Main entry point

use anyhow::Result;
use metamcp::{api, AuthService, Config, Database};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting MetaMCP server...");

    // Connect to database
    tracing::info!("Connecting to database...");
    let db = Database::new(&config.database_url).await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    db.run_migrations().await?;

    // Initialize auth service
    let auth_service = Arc::new(AuthService::new(
        config.jwt_secret.clone(),
        &config.encryption_key,
        db.clone(),
    ));

    // Create application state
    let state = api::AppState {
        db,
        auth: auth_service,
    };

    // Create router
    let app = api::create_router(state);

    // Bind and serve
    let bind_addr = config.bind_address();
    tracing::info!("Starting server on http://{}", bind_addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
