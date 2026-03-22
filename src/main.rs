use axum::{routing::post, Router};
use std::sync::Arc;

mod config;
mod db;
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

    let pool = db::build_pool(&config.sqlite_path, config.db_pool_size)?;
    db::run_migrations(&pool)?;

    let publisher = queue::publisher::Publisher::connect(&config).await?;

    let consumer_pool = pool.clone();
    let consumer_config = config.clone();
    tokio::spawn(async move {
        if let Err(e) = queue::consumer::start_consumer(consumer_pool, consumer_config).await {
            tracing::error!("consumer encerrou com erro: {e}");
        }
    });

    let app = Router::new()
        .route("/telemetry", post(handlers::telemetry::post_telemetry))
        .with_state(Arc::new(publisher));

    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
