#![recursion_limit = "128"]

extern crate openssl;
#[macro_use]
extern crate diesel;

mod app;
mod db;
mod s3;
mod schema;
mod serde;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::app::{config::AppConfig, http};
use tracing::warn;

#[tokio::main]
async fn main() {
    env_logger::init();

    warn!(version = %APP_VERSION, "Launching storage");

    let config = AppConfig::load().expect("cannot load config");

    http::run(config).await;
}
