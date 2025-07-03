use axum::{
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod handlers;
mod preview;
mod types;

use handlers::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "filepreview_rust=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize shared state for job management
    let jobs_state = Arc::new(RwLock::new(HashMap::new()));

    // Build our application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/preview", post(generate_preview))
        .route("/preview/async", post(generate_preview_async))
        .route("/preview/status/:job_id", get(get_job_status))
        .route("/download/:filename", get(download_file))
        .with_state(jobs_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Starting file preview service on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}