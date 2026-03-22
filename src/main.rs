use axum::{routing::post, Router};
use std::sync::Arc;

mod config;
mod error;
mod handlers;
mod models;
mod queue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env()?;
    let publisher = queue::publisher::Publisher::connect(&config).await?;

    let app = Router::new()
        .route("/telemetry", post(handlers::telemetry::post_telemetry))
        .with_state(Arc::new(publisher));

    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
