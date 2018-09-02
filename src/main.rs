extern crate config;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde;
#[macro_use]
extern crate tower_web;
extern crate http;

use std::env;

mod app;
mod tool;

fn main() {
    env_logger::init();

    let key = env::var("AWS_ACCESS_KEY_ID").unwrap();
    let secret = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
    let endpoint = env::var("AWS_ENDPOINT").unwrap();
    let region = env::var("AWS_REGION").unwrap();

    let s3 = tool::s3::Client::new(
        &key,
        &secret,
        &region,
        &endpoint,
        ::std::time::Duration::from_secs(300),
    );

    app::run(s3);
}
