use crate::app::{config::AppConfig, http};
use ::tracing::warn;

mod app;
mod s3;
mod serde;
mod tracing;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = tracing::init()?;

    warn!(version = %APP_VERSION, "Launching storage");

    let config = AppConfig::load().expect("cannot load config");
    warn!("config = {:?}", config);

    http::run(config).await;

    Ok(())
}
