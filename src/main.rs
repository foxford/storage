#![recursion_limit = "128"]

extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate tower_web;

use svc_authz::cache::{create_pool2, Cache};

fn main() {
    env_logger::init();
    use std::env::var;

    let db = var("DATABASE_URL")
        .map(|url| {
            let size = var("DATABASE_POOL_SIZE")
                .map(|val| {
                    val.parse::<u32>()
                        .expect("Error converting DATABASE_POOL_SIZE variable into u32")
                })
                .unwrap_or_else(|_| 5);
            let timeout = var("DATABASE_POOL_TIMEOUT")
                .map(|val| {
                    val.parse::<u64>()
                        .expect("Error converting DATABASE_POOL_TIMEOUT variable into u64")
                })
                .unwrap_or_else(|_| 5);

            crate::db::create_pool(&url, size, timeout)
        })
        .ok();

    let cache = var("CACHE_ENABLED")
        .ok()
        .and_then(|val| match val.as_ref() {
            "1" => {
                let url = var("CACHE_URL").unwrap_or_else(|_| panic!("Missing CACHE_URL variable"));

                let size = var("CACHE_POOL_SIZE")
                    .map(|val| {
                        val.parse::<u32>()
                            .expect("Error converting CACHE_POOL_SIZE variable into u32")
                    })
                    .unwrap_or_else(|_| 5);
                let idle_size = var("CACHE_POOL_IDLE_SIZE")
                    .map(|val| {
                        val.parse::<u32>()
                            .expect("Error converting CACHE_POOL_IDLE_SIZE variable into u32")
                    })
                    .ok();
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

                Some(Cache::new(
                    create_pool2(&url, size, idle_size, timeout),
                    expiration_time,
                ))
            }
            _ => None,
        });

    app::run(db, cache);
}

mod app;
mod db;
mod s3;
mod schema;
mod serde;
