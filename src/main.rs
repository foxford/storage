#[macro_use]
extern crate tower_web;

use svc_authz::cache::{create_pool, Cache};

fn main() {
    env_logger::init();

    use std::env::var;

    let cache = var("CACHE_URL").ok().map(|url| {
        let size = var("CACHE_POOL_SIZE")
            .map(|val| {
                val.parse::<u32>()
                    .expect("Error converting CACHE_POOL_SIZE variable into u32")
            })
            .unwrap_or_else(|_| 5);
        let timeout = var("CACHE_POOL_TIMEOUT")
            .map(|val| {
                val.parse::<u64>()
                    .expect("Error converting CACHE_POOL_TIMEOUT variable into u64")
            })
            .unwrap_or_else(|_| 5);
        let expiration_time = var("CACHE_EXPIRATION_TIME")
            .map(|val| {
                val.parse::<u64>()
                    .expect("Error converting CACHE_EXPIRATION_TIME variable into u64")
            })
            .unwrap_or_else(|_| 300);

        Cache::new(create_pool(&url, size, timeout), expiration_time)
    });

    let authz_wo = var("AUTHZ_WRITE_ONLY").ok() != None;

    app::run(cache, authz_wo);
}

mod app;
mod s3;
mod serde;
