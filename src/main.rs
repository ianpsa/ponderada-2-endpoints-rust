mod config;
mod error;
mod models;
mod queue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env()?;
    let _publisher = queue::publisher::Publisher::connect(&config).await?;

    tracing::info!("telemetry collector starting...");

    Ok(())
}
