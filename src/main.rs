#[macro_use]
extern crate tower_web;

use svc_authz::cache::{create_pool, Cache};

fn main() {
    env_logger::init();

    use std::env::var;
    let key = var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be specified");
    let secret = var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be specified");
    let endpoint = var("AWS_ENDPOINT").expect("AWS_ENDPOINT must be specified");
    let region = var("AWS_REGION").expect("AWS_REGION must be specified");

    let s3 = crate::s3::Client::new(
        &key,
        &secret,
        &region,
        &endpoint,
        ::std::time::Duration::from_secs(300),
    );

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

    app::run(s3, cache);
}

mod app;
mod s3;
mod serde;
