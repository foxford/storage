use crate::app::{config::AppConfig, http};
use tracing::warn;

mod app;
mod s3;
mod serde;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    env_logger::init();

    warn!(version = %APP_VERSION, "Launching storage");

    let config = AppConfig::load().expect("cannot load config");
    warn!("config = {:?}", config);

    http::run(config).await;
}
